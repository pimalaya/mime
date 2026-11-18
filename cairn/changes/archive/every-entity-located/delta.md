---
cairn: delta
change: every-entity-located
---

## ADDED Requirements

### Requirement: Every entity is located in the bytes

`MimeCst::spans` SHALL give, for every entity in the order of `MimeCst::entities`, the byte ranges of the whole entity, its header block (its empty end line included) and its body in the bytes `to_bytes` returns, which are the input of an unedited parse, an unparsed body counting as a leaf. A part's range SHALL exclude its delimiter lines and the line ending before the next one. The spans SHALL be computed from the tree on demand, the parse storing nothing more. `MimeCst::sections` SHALL walk the tree in IMAP part-number order (RFC 9051 6.4.5): the message first under the empty number, a multipart numbering its parts from 1, a message part numbering the parts of the message it holds, and a message whose body is no multipart holding one part numbered 1, its own entity. Each section SHALL carry its entity's span (`BODY[n]`, `BODY[n.MIME]`) and, for the message itself and a parsed `message/rfc822` or `message/global` part, the span of the message it holds (`BODY[n.HEADER]`, `BODY[n.TEXT]`).

#### Scenario: The corpus sliced by its spans
- GIVEN every vendored fixture, as written and in its LF and CRLF forms
- WHEN each is parsed and its spans taken
- THEN each span slices its entity's serialized bytes out of the input, agrees with the offsets held to mail-parser's frozen answers, and, on every input parsed with no anomaly, agrees with a split on the boundary lines and the first empty line alone

#### Scenario: Nested multiparts and messages numbered
- GIVEN a `multipart/mixed` holding a text part, a `multipart/alternative` of two parts, a `message/rfc822` holding a multipart of two parts, and a `message/rfc822` holding a single-part message
- WHEN its sections are taken
- THEN they are numbered the empty number, `1`, `2`, `2.1`, `2.2`, `3`, `3.1`, `3.2`, `4`, `4.1`, and each range slices the bytes IMAP names

## MODIFIED Requirements

## REMOVED Requirements
