---
cairn: change
id: a-message-is-never-encoded
status: landed
created: 2026-11-12
---

# A message is never encoded

## Why

RFC 2046 5.2.1 allows no encoding but `7bit`, `8bit` or `binary` for a `message/rfc822` body, yet the builder wrote an attached message base64 whenever no identity encoding fit as given: a message file with LF line breaks under a CRLF build, or any 8-bit message under the default options. Himalaya's live tests found it on `message compose --attach mail.eml` and on forwards. Once a CRLF message did fit, a second bug showed: the boundary fast path did not count a message leaf as content written as is, so attaching a message this builder produced reused its `=_mime-rs_` boundary and broke the outer structure.

## What

- A `message/rfc822` or `message/global` leaf gets its line breaks written as the build's line ending, as a text leaf does, before the identity encodings are tried.
- When none fits, its content is parsed and written as the nested message it holds, its own fields and leaves encoded by the building rules. Content opening with no field, or a build without the parser, keeps the old fallback.
- The boundary fast path counts a message leaf as written as is, so content holding the generated prefix takes the checked path.
