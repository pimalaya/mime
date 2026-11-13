---
cairn: change
id: the-first-occurrence-is-read
status: landed
created: 2026-11-13
---

# The first occurrence is read

## Why

When an entity stated one of its structural fields twice, this crate read the last occurrence, as mail-parser 1.0 does. The pimdir standard, which io-pimdir implements over this crate, reads a field stated twice from its first occurrence everywhere (STORAGE Annex A.1: "a field present twice is read from its first occurrence, in document order", and an entity's `Content-Type`, `Content-Disposition`, `Content-Transfer-Encoding` and `Content-ID` "are read as a header field is"). io-pimdir therefore cut the later occurrences out of a message before handing it over (its `summary::mail::first_occurrences`), a pass over every message that exists only to undo this crate's choice. The owner decided on 2026-11-13 that this crate reads the first.

The last was also not read everywhere: the typed read of a field (`MimeEntity::header`, `Mime::header`) and the field edits (`set`, and the Content-Transfer-Encoding a content edit rewrites) already took the first, so a leaf stating two encodings had one rewritten and the other read.

## What

- The parse reads an entity's Content-Type (the kind of its body, its `boundary`) and Content-Transfer-Encoding (whether a message nests) from their first occurrence.
- `MimeEntity::content_type`, `transfer_encoding`, `disposition` and `attachment_name`, and through them the decoding and charset of a leaf and the body lists, read the first.
- `Mime::content_type` reads the first, and the writer takes the declared type, the declared encoding and the `filename` of a legacy `name` from the first, writing its one Content-Type and one Content-Transfer-Encoding where the first stood.
- A regenerated boundary is written into the first Content-Type.
- Content-ID is unchanged: its only reads are the typed ones, which took the first already.
- The differential oracle records the difference with mail-parser as a named rule, `RepeatedField`, and its helpers look at the field this crate reads.
- The fixture of the pimdir vectors that separates the two readings is vendored under tests/corpus/pimdir-vectors.
