//! Spans and IMAP sections.
//!
//! On every fixture, as written and in its LF and CRLF forms, the span of
//! each entity slices exactly its bytes out of the input and agrees with the
//! offsets mail-parser's frozen answers are held to; on every fixture parsed
//! with no anomaly, the spans agree with an independent split on the
//! boundaries and the empty lines.

#![cfg(feature = "parser")]

mod common;

use mime::tree::{body::MimeBodyNode, cst::MimeCst, entity::MimeEntity, span::MimeSpan};

use crate::common::{fixtures, line_endings, shapes};

const CORPORA: [(&str, usize); 7] = [
    ("dovecot-malformed", 24),
    ("dovecot-thirdparty", 20),
    ("pimdir-vectors", 1),
    ("stalwart-issues", 18),
    ("stalwart-regressions", 75),
    ("stalwart-rfc", 10),
    ("stalwart-sieve", 6),
];

fn concat(chunks: Vec<&[u8]>) -> Vec<u8> {
    chunks.concat()
}

fn header_bytes(entity: &MimeEntity<'_>) -> Vec<u8> {
    let mut whole = entity.to_bytes();
    whole.truncate(entity.header.len());
    whole
}

/// Checks every span against the tree and the shapes of the corpus harness.
fn spans_slice_the_input(name: &str, input: &[u8]) {
    let cst = MimeCst::parse(input);
    let entities = cst.entities();
    let spans = cst.spans();
    assert_eq!(spans.len(), entities.len(), "{name}");
    assert_eq!(spans[0].entity, 0..input.len(), "{name}");

    for (index, (entity, span)) in entities.iter().zip(&spans).enumerate() {
        assert_eq!(span.header.start, span.entity.start, "{name} #{index}");
        assert_eq!(span.header.end, span.body.start, "{name} #{index}");
        assert_eq!(span.body.end, span.entity.end, "{name} #{index}");
        assert_eq!(
            &input[span.entity.clone()],
            concat(entity.chunks()),
            "{name} #{index}"
        );
        assert_eq!(
            &input[span.header.clone()],
            header_bytes(entity),
            "{name} #{index}"
        );
    }

    for (shape, span) in shapes(&cst).iter().zip(&spans) {
        assert_eq!(shape.offset_header, span.header.start, "{name}");
        assert_eq!(shape.offset_body, span.body.start, "{name}");
        assert_eq!(shape.offset_end, span.entity.end, "{name}");
    }
}

/// The header block of `entity`, found by the first empty line alone.
fn split_header(entity: &[u8]) -> usize {
    if entity.starts_with(b"\n") {
        return 1;
    }

    if entity.starts_with(b"\r\n") {
        return 2;
    }

    for at in 0..entity.len() {
        if entity[at] != b'\n' {
            continue;
        }

        if entity[at + 1..].starts_with(b"\n") {
            return at + 2;
        }

        if entity[at + 1..].starts_with(b"\r\n") {
            return at + 3;
        }
    }

    entity.len()
}

/// The parts of a multipart body, found by its delimiter lines alone.
///
/// Each range is relative to `body`, its delimiter lines and the line
/// ending before the next one excluded.
fn split_parts(body: &[u8], boundary: &[u8]) -> Vec<std::ops::Range<usize>> {
    let dash = [b"--".as_slice(), boundary].concat();
    let mut delimiters = Vec::new();

    for at in 0..body.len() {
        let line_start = at == 0 || body[at - 1] == b'\n';

        if !line_start || !body[at..].starts_with(&dash) {
            continue;
        }

        let rest = &body[at + dash.len()..];
        let close = rest.starts_with(b"--");

        if close || matches!(rest.first(), None | Some(b'\r' | b'\n' | b' ' | b'\t')) {
            delimiters.push((at, close));

            if close {
                break;
            }
        }
    }

    let mut parts = Vec::new();

    for pair in delimiters.windows(2) {
        let (open, _) = pair[0];
        let (next, _) = pair[1];
        let start = body[open..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(body.len(), |eol| open + eol + 1);
        let mut end = next;

        if end > start && body[end - 1] == b'\n' {
            end -= 1;

            if end > start && body[end - 1] == b'\r' {
                end -= 1;
            }
        }

        parts.push(start..end.max(start));
    }

    parts
}

/// Checks the spans of an anomaly-free parse against the naive splits.
fn spans_agree_with_a_naive_split(name: &str, input: &[u8]) -> bool {
    let cst = MimeCst::parse(input);

    if !cst.anomalies().is_empty() {
        return false;
    }

    let entities = cst.entities();
    let spans = cst.spans();
    let index_of = |entity: &MimeEntity<'_>| {
        entities
            .iter()
            .position(|other| std::ptr::eq(*other, entity))
            .expect("entity")
    };

    for (entity, span) in entities.iter().zip(&spans) {
        let start = span.entity.start;
        let header = split_header(&input[span.entity.clone()]);
        assert_eq!(span.header, start..start + header, "{name} header");

        match &entity.body {
            MimeBodyNode::Multipart(multipart) => {
                let boundary = entity.content_type().param("boundary").expect("boundary");
                let body = &input[span.body.clone()];
                let naive = split_parts(body, boundary.as_bytes());
                assert_eq!(naive.len(), multipart.parts.len(), "{name} parts");

                for (part, range) in multipart.parts.iter().zip(naive) {
                    let ours = &spans[index_of(&part.entity)];
                    let theirs = span.body.start + range.start..span.body.start + range.end;
                    assert_eq!(ours.entity, theirs, "{name} part");
                }
            }
            MimeBodyNode::Message(nested) => {
                let ours = &spans[index_of(nested)];
                assert_eq!(ours.entity, span.body, "{name} message");
            }
            MimeBodyNode::Leaf(_) | MimeBodyNode::Unparsed(_) => {}
        }
    }

    true
}

#[test]
fn every_span_slices_its_entity_out_of_the_input() {
    for (corpus, count) in CORPORA {
        for fixture in fixtures(corpus, count) {
            let (lf, crlf) = line_endings(&fixture.bytes);
            spans_slice_the_input(&fixture.name, &fixture.bytes);
            spans_slice_the_input(&format!("{} (lf)", fixture.name), &lf);
            spans_slice_the_input(&format!("{} (crlf)", fixture.name), &crlf);
        }
    }
}

#[test]
fn spans_agree_with_an_independent_split() {
    let mut checked = 0;
    let mut nested = 0;

    for (corpus, count) in CORPORA {
        for fixture in fixtures(corpus, count) {
            let (lf, crlf) = line_endings(&fixture.bytes);

            for input in [fixture.bytes.as_slice(), &lf, &crlf] {
                if spans_agree_with_a_naive_split(&fixture.name, input) {
                    checked += 1;

                    let cst = MimeCst::parse(input);
                    let entities = cst.entities();

                    if entities
                        .iter()
                        .any(|entity| matches!(entity.body, MimeBodyNode::Message(_)))
                    {
                        nested += 1;
                    }
                }
            }
        }
    }

    assert!(checked >= 300, "{checked} inputs checked");
    assert!(nested >= 40, "{nested} inputs with a nested message");
}

const NESTED: &str = concat!(
    "From: a@example.com\r\n",
    "Subject: outer\r\n",
    "Content-Type: multipart/mixed; boundary=\"outer\"\r\n",
    "\r\n",
    "preamble\r\n",
    "--outer\r\n",
    "Content-Type: text/plain\r\n",
    "\r\n",
    "first\r\n",
    "line two\r\n",
    "--outer\r\n",
    "Content-Type: multipart/alternative; boundary=\"inner\"\r\n",
    "\r\n",
    "--inner\r\n",
    "Content-Type: text/plain\r\n",
    "\r\n",
    "plain\r\n",
    "--inner\r\n",
    "Content-Type: text/html\r\n",
    "\r\n",
    "<p>html</p>\r\n",
    "--inner--\r\n",
    "--outer\r\n",
    "Content-Type: message/rfc822\r\n",
    "\r\n",
    "Subject: attached\r\n",
    "Content-Type: multipart/mixed; boundary=\"deep\"\r\n",
    "\r\n",
    "--deep\r\n",
    "\r\n",
    "deep one\r\n",
    "--deep\r\n",
    "Content-Type: application/octet-stream\r\n",
    "\r\n",
    "deep two\r\n",
    "--deep--\r\n",
    "--outer\r\n",
    "Content-Type: message/rfc822\r\n",
    "\r\n",
    "Subject: single\r\n",
    "\r\n",
    "single body\r\n",
    "--outer--\r\n",
    "epilogue\r\n",
);

fn slice<'i>(input: &'i [u8], range: &std::ops::Range<usize>) -> &'i str {
    std::str::from_utf8(&input[range.clone()]).expect("utf-8")
}

#[test]
fn sections_follow_imap_part_numbers() {
    let input = NESTED.as_bytes();
    let cst = MimeCst::parse(input);
    assert!(cst.anomalies().is_empty());

    let sections = cst.sections();
    let numbers: Vec<Vec<usize>> = sections.iter().map(|s| s.part.clone()).collect();
    assert_eq!(
        numbers,
        [
            vec![],
            vec![1],
            vec![2],
            vec![2, 1],
            vec![2, 2],
            vec![3],
            vec![3, 1],
            vec![3, 2],
            vec![4],
            vec![4, 1],
        ]
    );

    let at = |part: &[usize]| {
        sections
            .iter()
            .find(|section| section.part == part)
            .expect("section")
    };

    let root = at(&[]);
    assert_eq!(root.span.entity, 0..input.len());
    assert!(slice(input, &root.span.header).ends_with("boundary=\"outer\"\r\n\r\n"));
    assert_eq!(root.message.as_ref(), Some(&root.span));

    let first = at(&[1]);
    assert_eq!(
        slice(input, &first.span.header),
        "Content-Type: text/plain\r\n\r\n"
    );
    assert_eq!(slice(input, &first.span.body), "first\r\nline two");
    assert_eq!(first.message, None);

    let html = at(&[2, 2]);
    assert_eq!(slice(input, &html.span.body), "<p>html</p>");

    let attached = at(&[3]);
    assert_eq!(
        slice(input, &attached.span.header),
        "Content-Type: message/rfc822\r\n\r\n"
    );
    let message = attached.message.as_ref().expect("message");
    assert_eq!(message.entity, attached.span.body);
    assert!(slice(input, &message.header).starts_with("Subject: attached\r\n"));
    assert!(slice(input, &message.body).starts_with("--deep\r\n"));
    assert!(slice(input, &message.body).ends_with("--deep--"));

    let deep = at(&[3, 1]);
    assert_eq!(slice(input, &deep.span.header), "\r\n");
    assert_eq!(slice(input, &deep.span.body), "deep one");
    assert_eq!(slice(input, &at(&[3, 2]).span.body), "deep two");

    let single = at(&[4]);
    let message = single.message.as_ref().expect("message");
    assert_eq!(slice(input, &message.header), "Subject: single\r\n\r\n");
    assert_eq!(slice(input, &message.body), "single body");

    let inner = at(&[4, 1]);
    assert_eq!(inner.span, *message);
    assert_eq!(slice(input, &inner.span.body), "single body");
}

#[test]
fn a_single_part_message_has_part_one() {
    let input = b"Subject: hi\nContent-Type: text/plain\n\nhello\n";
    let cst = MimeCst::parse(input);
    let sections = cst.sections();
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].part, Vec::<usize>::new());
    assert_eq!(sections[1].part, [1]);

    let span = MimeSpan {
        entity: 0..input.len(),
        header: 0..38,
        body: 38..input.len(),
    };
    assert_eq!(sections[1].span, span);
    assert_eq!(sections[1].message, None);
    assert_eq!(&input[span.body], b"hello\n");
}

#[test]
fn spans_index_the_serialized_bytes_after_an_edit() {
    use mime::{build::MimeBuildOptions, header::marker::SUBJECT};

    let mut cst = MimeCst::parse(NESTED.as_bytes());
    let opts = MimeBuildOptions::default();
    cst.header_mut(0)
        .expect("root header")
        .set::<SUBJECT>("a much longer subject".into(), &opts)
        .expect("edit");

    let bytes = cst.to_bytes();

    for (entity, span) in cst.entities().iter().zip(cst.spans()) {
        assert_eq!(bytes[span.entity], concat(entity.chunks()));
    }
}
