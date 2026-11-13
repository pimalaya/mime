---
cairn: log
change: the-first-occurrence-is-read
landed: 2026-11-13
---

# The first occurrence is read

Landed [the-first-occurrence-is-read](../changes/archive/the-first-occurrence-is-read/proposal.md), decided by the owner on 2026-11-13: the pimdir standard reads a field stated twice from its first occurrence (STORAGE Annex A.1), and io-pimdir cut the later occurrences out of every message before handing it to this crate, which read the last as mail-parser does.

A Content-Type, a Content-Transfer-Encoding or a Content-Disposition an entity states twice is now read from its first occurrence, in document order, wherever the crate reads it: the parse (the kind of a body, its boundary, whether a message nests), the entity accessors and through them the decoding of a leaf, its charset and the body lists, `Mime::content_type`, the type, encoding and legacy name the writer takes from a model, and the Content-Type a regenerated boundary is written into. Content-ID has no read but the typed ones, which took the first already, as did the field edits: a leaf stating two encodings had its first rewritten by `replace_content` and its last read, which this change settles.

Code: `Kind::of` and `identity_encoding` in src/tree/parse.rs, `content_type`, `content`, `transfer_encoding` and `disposition` in src/tree/entity.rs, `rewrite_boundary` in src/tree/edit/structure.rs, `Mime::content_type` in src/mime.rs, `first` (was `last`) in src/build/entity.rs. No read allocates, and each but the one walk of `content` stops at the first field of its name. Nothing was timed: the change is one of accuracy.

The differential oracle gained its 40th rule, `RepeatedField`: mail-parser 1.0 reads the last of the three fields, seen on a message for each (the boundary, the charset, the encoding, the disposition). It holds only where an entity states one of them twice with a last value that is not its first, on the structure (Content-Type or Content-Transfer-Encoding, anywhere in the message, as the rules beside it), on a leaf's content and problems (its own two fields) and on the body lists (Content-Type or Content-Disposition, anywhere). The helpers that looked at the last Content-Type, Content-Transfer-Encoding and Content-Disposition look at the first, the one read. Of the 27,359 stored inputs of the fuzz crate (the `differential` and `parse` corpora and the differential artifacts), swept through tests/fetched.rs, all read alike by rule, 861 of them by this one.

Three vendored fixtures state a Content-Type twice in one part and moved: dovecot-malformed 007 (its second part is now a multipart holding no part), 020 and 021 (a `multipart/alternative` where a text part was read). Each has its structure rows in tests/corpus/divergence.tsv; the four text rows of 020 and 021 went, their part being no text any more. The fixture mail-repeated-part-fields.eml of the pimdir vectors is vendored under tests/corpus/pimdir-vectors, byte for byte, and swept with the others; it has no frozen answer and is the one fixture not held to mail-parser's offsets.

Tests, each seen failing with the old read put back, alone and together: the fixture's tree, a Content-Type, a Content-Transfer-Encoding and a Content-Disposition stated twice (tests/decoded.rs), a model stating them twice written from its first (tests/build.rs), a boundary regenerated in the first Content-Type and a leaf replaced in the encoding read (tests/edit.rs), and the oracle letting each difference through by rule while seeing it without (tests/differential.rs). A Content-ID test pins what already held.

Left as found: the validator reports no Content- field stated twice (its repetition rule covers the fields a part may not carry), so the pimdir fixture checks valid; and benches/compare.rs reads every directory of tests/corpus, so its workload gained this fixture and benches/README.md still counts 153.

Capabilities moved: parsing, building, editing.
