---
cairn: change
id: every-entity-located
status: landed
created: 2026-11-18
---

# Every entity located

## Why

Pimalaya Server stores a message as its raw bytes and answers IMAP `BODY[n]`, `BODY[n.MIME]`, `BODY[n.HEADER]`, `BODY[n.TEXT]`, partial fetches and BODYSTRUCTURE sizes and line counts from them. It needs the byte range of every part, its header block and its body in the original input, numbered as IMAP numbers parts, without parsing the message again per fetch. The tree held no offset: only the corpus harness of the tests summed leaf lengths to compare with mail-parser.

## What

`MimeCst::spans` and `MimeCst::sections` in a new `tree::span` module, and `MimeHeaderBlock::len`. The ranges are summed from the leaves on demand, so the parse and its tree are unchanged.
