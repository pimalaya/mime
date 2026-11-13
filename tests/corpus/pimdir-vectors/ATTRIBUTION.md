# pimdir vectors

A message from the test vectors of the [pimdir](https://github.com/pimalaya/pimdir) standard.

- Source: vectors/fixtures/ of pimdir, mail-repeated-part-fields.eml (SHA-256 `5e243c8aeae365b9fa9dd5c15fe5ae372f474a568f7876903d9f99a5ff1286d6`, as vectors/manifest.json lists it). No frozen answers.
- Licence: MIT OR Apache-2.0, Copyright (c) 2026 Clément DOUIN (soywod). The file keeps that licence.
- Why: it separates the two readings of a structural field an entity states twice. The message states Content-Type twice and its second part states Content-Type, Content-Disposition and Content-Transfer-Encoding twice each: read from the first occurrence, as pimdir and this crate read them, it is a multipart whose second part is the attachment first.csv; read from the last, as mail-parser reads them, one text part.

Nothing here is edited. tests/decoded.rs states the tree this crate reads, and the differential oracle lets mail-parser's other reading through by its `RepeatedField` rule.
