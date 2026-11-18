//! # Spans
//!
//! Where every entity of a tree sits in the bytes it serializes to, which
//! are the input of an unedited parse: the whole entity, its header block
//! and its body, as byte ranges, so a store keeping the raw message serves
//! any part of it without parsing it again.
//!
//! [`MimeCst::sections`] walks the tree in IMAP part-number order (RFC 3501
//! 6.4.5, RFC 9051 6.4.5), the ranges of a `BODY[n]`, `BODY[n.MIME]`,
//! `BODY[n.HEADER]` and `BODY[n.TEXT]` fetch on each section.

use alloc::vec::Vec;
use core::ops::Range;

use crate::tree::{body::MimeBodyNode, cst::MimeCst, entity::MimeEntity};

/// Where one entity sits in the serialized message.
///
/// The header block includes its empty end line; the body runs from it to
/// the end of the entity, so `header.end == body.start` and the two cover
/// `entity`. A part's range excludes its delimiter lines and the line
/// ending before the next one (RFC 2046 5.1.1).
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct MimeSpan {
    /// The whole entity.
    pub entity: Range<usize>,
    /// Its header block.
    pub header: Range<usize>,
    /// Its body.
    pub body: Range<usize>,
}

/// One section of a message, numbered as IMAP numbers parts.
///
/// Refs: <https://datatracker.ietf.org/doc/html/rfc9051#section-6.4.5>
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MimeSection<'s, 'a> {
    /// The part number, each level from 1; empty for the message itself.
    pub part: Vec<usize>,
    /// The entity the part number names.
    pub entity: &'s MimeEntity<'a>,
    /// Where it sits: `BODY[n]` is its body, `BODY[n.MIME]` its header.
    pub span: MimeSpan,
    /// The message the part holds, for `BODY[n.HEADER]` and `BODY[n.TEXT]`.
    ///
    /// The message itself for the empty part number, the nested message of
    /// a parsed `message/rfc822` or `message/global` part, `None` for any
    /// other part.
    pub message: Option<MimeSpan>,
}

impl<'a> MimeCst<'a> {
    /// The span of every entity, in the order of [`entities`](Self::entities).
    ///
    /// Ranges index [`to_bytes`](Self::to_bytes), which is the input while
    /// the tree is unedited. An unparsed body counts as a leaf.
    pub fn spans(&self) -> Vec<MimeSpan> {
        let mut out = Vec::with_capacity(8);
        let mut offset = 0;
        push_spans(&self.root, &mut offset, &mut out);
        out
    }

    /// Every section, in IMAP part-number order.
    ///
    /// The message first, with an empty part number. A multipart numbers
    /// its parts from 1 and a message part numbers the parts of the message
    /// it holds, while a message whose body is no multipart, the root
    /// included, holds one part numbered 1: its own entity. Expand an
    /// unparsed tree first, an unparsed body holding no section.
    pub fn sections(&self) -> Vec<MimeSection<'_, 'a>> {
        let spans = self.spans();
        let mut out = Vec::with_capacity(spans.len() + 1);
        let root = &self.root;

        out.push(MimeSection {
            part: Vec::new(),
            entity: root,
            span: spans[0].clone(),
            message: Some(spans[0].clone()),
        });

        let mut index = 0;
        let mut part = Vec::new();
        push_message(root, &spans, &mut index, &mut part, &mut out);
        out
    }
}

/// Pushes the span of `entity`, then of every entity under it.
fn push_spans(entity: &MimeEntity<'_>, offset: &mut usize, out: &mut Vec<MimeSpan>) {
    let start = *offset;
    let index = out.len();
    *offset += entity.header.len();
    let body = *offset;

    out.push(MimeSpan {
        entity: start..start,
        header: start..body,
        body: body..body,
    });

    match &entity.body {
        MimeBodyNode::Leaf(bytes) => *offset += bytes.len(),
        MimeBodyNode::Unparsed(unparsed) => *offset += unparsed.bytes.len(),
        MimeBodyNode::Multipart(multipart) => {
            *offset += multipart.preamble.len();

            for part in &multipart.parts {
                *offset += part.delimiter.len();
                push_spans(&part.entity, offset, out);
            }

            *offset += multipart.close.len() + multipart.epilogue.len();
        }
        MimeBodyNode::Message(nested) => push_spans(nested, offset, out),
    }

    out[index].entity.end = *offset;
    out[index].body.end = *offset;
}

/// Pushes the sections of the message `entity`, the span at `index` its own.
///
/// `index` ends past every entity under it.
fn push_message<'s, 'a>(
    entity: &'s MimeEntity<'a>,
    spans: &[MimeSpan],
    index: &mut usize,
    part: &mut Vec<usize>,
    out: &mut Vec<MimeSection<'s, 'a>>,
) {
    if matches!(entity.body, MimeBodyNode::Multipart(_)) {
        push_parts(entity, spans, index, part, out);
    } else {
        part.push(1);
        push_part(entity, spans, index, part, out);
        part.pop();
    }
}

/// Pushes the section of the part `entity` numbered `part`, then its own.
fn push_part<'s, 'a>(
    entity: &'s MimeEntity<'a>,
    spans: &[MimeSpan],
    index: &mut usize,
    part: &mut Vec<usize>,
    out: &mut Vec<MimeSection<'s, 'a>>,
) {
    let message = match entity.body {
        MimeBodyNode::Message(_) => spans.get(*index + 1).cloned(),
        _ => None,
    };

    out.push(MimeSection {
        part: part.clone(),
        entity,
        span: spans[*index].clone(),
        message,
    });

    push_parts(entity, spans, index, part, out);
}

/// Pushes the sections under the entity at `index`, moving past them.
fn push_parts<'s, 'a>(
    entity: &'s MimeEntity<'a>,
    spans: &[MimeSpan],
    index: &mut usize,
    part: &mut Vec<usize>,
    out: &mut Vec<MimeSection<'s, 'a>>,
) {
    match &entity.body {
        MimeBodyNode::Multipart(multipart) => {
            for (number, child) in multipart.parts.iter().enumerate() {
                *index += 1;
                part.push(number + 1);
                push_part(&child.entity, spans, index, part, out);
                part.pop();
            }
        }
        MimeBodyNode::Message(nested) => {
            *index += 1;
            push_message(nested, spans, index, part, out);
        }
        MimeBodyNode::Leaf(_) | MimeBodyNode::Unparsed(_) => {}
    }
}
