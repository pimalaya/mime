---
cairn: tasks
change: the-first-occurrence-is-read
---

- [x] The parse reads the first Content-Type and the first Content-Transfer-Encoding (`Kind::of` and `identity_encoding` in src/tree/parse.rs)
- [x] The entity accessors read the first (`content_type`, `content`, `transfer_encoding`, `disposition` in src/tree/entity.rs)
- [x] The model and the writer read the first (`Mime::content_type` in src/mime.rs, `first` in src/build/entity.rs)
- [x] A regenerated boundary goes to the first Content-Type (`rewrite_boundary` in src/tree/edit/structure.rs)
- [x] The oracle's `RepeatedField` rule, on the structure, the content and the body lists, and its helpers reading the first
- [x] The pimdir fixture vendored under tests/corpus/pimdir-vectors, swept by tests/corpus.rs, tests/differential.rs and tests/validate.rs
- [x] Divergence rows for the three vendored fixtures whose structure moved (dovecot-malformed 007, 020, 021), the four text rows of 020 and 021 that no longer compare removed
- [x] Tests in tests/decoded.rs, tests/build.rs, tests/edit.rs and tests/differential.rs, each seen failing with the old read put back, but the Content-ID one, which pins what already held
- [x] Fold into cairn/spec (parsing, building, editing), log, CHANGELOG
