---
cairn: delta
change: the-first-occurrence-is-read
---

## ADDED Requirements

### Requirement: A structural field stated twice is read from its first occurrence

When an entity states its Content-Type, Content-Transfer-Encoding, Content-Disposition or Content-ID more than once, every read of that field SHALL take its first occurrence, in document order, as `MimeEntity::header` reads any field: the parse (the kind of the body, its `boundary`, whether a message nests), `MimeEntity::content_type`, `transfer_encoding`, `disposition` and `attachment_name`, the decoding and the charset of a leaf, the body lists, and the field an edit rewrites. RFC 2045 3 gives an entity one of each and names no winner between two; the first is what the pimdir standard reads (STORAGE Annex A.1). `MimeHeaderBlock::get_all` SHALL still yield every occurrence. mail-parser 1.0 reads the last, which the differential oracle lets through by its `RepeatedField` rule.

#### Scenario: A message and a part stating their fields twice
- GIVEN tests/corpus/pimdir-vectors/mail-repeated-part-fields.eml, a message stating Content-Type twice (`multipart/mixed`, then `text/plain`) whose second part states Content-Type (`text/csv`, then `application/pdf`), Content-Disposition (`attachment` named `first.csv`, then `inline` named `second.pdf`) and Content-Transfer-Encoding (`base64`, then `7bit`) twice each
- WHEN it is parsed
- THEN the root is a multipart of two parts, and the second reads as `text/csv`, an attachment named `first.csv`, base64, of 4 decoded octets

## MODIFIED Requirements

### Requirement: Structure comes from the structural fields alone

Parsing SHALL read the Content-Type and the Content-Transfer-Encoding of each entity each from its first occurrence when the entity states it twice, where the last of each won.

### Requirement: The corpus is swept

The fixtures without frozen answers SHALL match mail-parser 1.0's offsets live, but the one of tests/corpus/pimdir-vectors, which states its structural fields twice and whose tree a test states.

### Requirement: What the writer normalises

A model stating a Content-Type, a Content-Transfer-Encoding or a Content-Disposition twice SHALL be read from the first of each, as `Mime::content_type` reads it and as a parsed entity is (parsing.md): the type and the encoding it declares, and the `filename` a legacy `name` repeats. The writer SHALL write one Content-Type and one Content-Transfer-Encoding, where the first stood.

### Requirement: Leaf content is re-encoded on replacement

`MimeCst::replace_content` SHALL rewrite or add the entity's Content-Transfer-Encoding field, the first when it states two, which is the one read (parsing.md), and no other.

### Requirement: Structural edits rewrite their multipart alone

A regenerated boundary SHALL be written into the multipart's boundary parameter, in the first Content-Type when it states two.

## REMOVED Requirements
