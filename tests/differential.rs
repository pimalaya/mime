//! The differential oracle (tests/common/differential.rs) over the vendored
//! corpus, mail-parser 1.0 run live: every fixture, as written and in its LF
//! and CRLF forms, reads alike through both but where a named rule lets a
//! deliberate difference through. The fuzz crate's `differential` target
//! runs the same oracle over arbitrary bytes, and tests/fetched.rs over the
//! fetched corpora.

#![cfg(all(feature = "parser", feature = "base64", feature = "encoding"))]

mod common;
#[path = "common/differential.rs"]
mod differential;

use mail_parser::MessageParser;
use serde_json::Value;

use crate::{
    common::{fixtures, line_endings},
    differential::Rule,
};

const CORPORA: [(&str, usize); 7] = [
    ("dovecot-malformed", 24),
    ("dovecot-thirdparty", 20),
    ("pimdir-vectors", 1),
    ("stalwart-issues", 18),
    ("stalwart-regressions", 75),
    ("stalwart-rfc", 10),
    ("stalwart-sieve", 6),
];

fn answer(input: &[u8]) -> Option<Value> {
    MessageParser::new()
        .parse(input)
        .map(|message| serde_json::to_value(&message).expect("serializes"))
}

#[test]
fn every_fixture_reads_as_mail_parser_reads_it_but_by_rule() {
    let mut failures = Vec::new();

    for (corpus, count) in CORPORA {
        for fixture in fixtures(corpus, count) {
            let (lf, crlf) = line_endings(&fixture.bytes);

            for (form, input) in [("as written", &fixture.bytes), ("lf", &lf), ("crlf", &crlf)] {
                let outcome = differential::compare(input, answer(input).as_ref(), false);

                for difference in outcome.differences {
                    failures.push(format!(
                        "{} ({form}) {}: ours {}, theirs {}",
                        fixture.name, difference.path, difference.ours, difference.theirs
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} differences:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn the_oracle_sees_a_part_text_or_list_read_otherwise() {
    let input = concat!(
        "Content-Type: multipart/mixed; boundary=b\r\n",
        "\r\n",
        "--b\r\n",
        "Content-Type: text/plain\r\n",
        "\r\n",
        "hello\r\n",
        "--b--\r\n",
    )
    .as_bytes();
    let answer = answer(input).expect("mail-parser reads it");
    let paths = |answer: &Value| -> Vec<String> {
        differential::compare(input, Some(answer), false)
            .differences
            .into_iter()
            .map(|difference| difference.path)
            .collect()
    };

    assert_eq!(paths(&answer), Vec::<String>::new());

    let mut moved = answer.clone();
    moved["parts"][1]["offset_end"] = Value::from(input.len() as u64);
    assert_eq!(paths(&moved), ["structure"]);

    let mut text = answer.clone();
    text["parts"][1]["text"] = Value::from("bye");
    assert_eq!(paths(&text), ["part[1].text"]);

    let mut lists = answer;
    lists["messages"][0]["text_body"] = Value::Array(Vec::new());
    assert_eq!(paths(&lists), ["text_body"]);
}

#[test]
fn a_structural_field_stated_twice_reads_otherwise_by_rule() {
    let fixture = fixtures("pimdir-vectors", 1).remove(0).bytes;
    let content_type = concat!(
        "Content-Type: multipart/mixed; boundary=a\r\n",
        "Content-Type: multipart/mixed; boundary=b\r\n",
        "\r\n",
        "--a\r\n",
        "\r\n",
        "one\r\n",
        "--b\r\n",
        "\r\n",
        "two\r\n",
        "--a--\r\n",
    );
    let charset = concat!(
        "Content-Type: text/plain; charset=iso-8859-1\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "café\r\n",
    );
    let transfer_encoding = concat!(
        "Content-Type: text/plain\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "Content-Transfer-Encoding: 7bit\r\n",
        "\r\n",
        "aGVsbG8=\r\n",
    );
    let disposition = concat!(
        "Content-Type: multipart/mixed; boundary=a\r\n",
        "\r\n",
        "--a\r\n",
        "\r\n",
        "See the file.\r\n",
        "--a\r\n",
        "Content-Type: image/png\r\n",
        "Content-Disposition: attachment; filename=\"first.png\"\r\n",
        "Content-Disposition: inline; filename=\"second.png\"\r\n",
        "\r\n",
        "data\r\n",
        "--a--\r\n",
    );

    for (name, input) in [
        ("the fixture", fixture.as_slice()),
        ("Content-Type", content_type.as_bytes()),
        ("charset", charset.as_bytes()),
        ("Content-Transfer-Encoding", transfer_encoding.as_bytes()),
        ("Content-Disposition", disposition.as_bytes()),
    ] {
        let answer = answer(input).expect("mail-parser reads it");
        let outcome = differential::compare(input, Some(&answer), false);

        assert!(
            outcome.differences.is_empty(),
            "{name}: {:#?}",
            outcome.differences
        );
        assert!(!outcome.rules.is_empty(), "{name}");
        assert!(
            outcome
                .rules
                .iter()
                .all(|rule| *rule == Rule::RepeatedField),
            "{name}: {:?}",
            outcome.rules
        );

        // NOTE: with no rule, the oracle sees the two readings differ.
        let strict = differential::compare(input, Some(&answer), true);
        assert!(!strict.differences.is_empty(), "{name}");
    }
}
