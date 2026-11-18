---
cairn: log
change: every-entity-located
landed: 2026-11-18
---

# Every entity located

Landed [every-entity-located](../changes/archive/every-entity-located/proposal.md), asked for by Pimalaya Server, which keeps a message as its raw bytes and serves IMAP fetches out of them.

The tree held no offset: every leaf borrows the input, and only the corpus harness of the tests summed leaf lengths to compare with mail-parser's offsets. `MimeCst::spans` now does that sum in the library, one walk giving for every entity, in `entities` order, the ranges of the whole entity, its header block (empty end line included) and its body in `to_bytes`, the input itself while the tree is unedited. `MimeCst::sections` walks the tree in IMAP part-number order (RFC 9051 6.4.5) over those spans: the message under the empty number, parts of a multipart from 1, a message part numbering the parts of the message it holds, a non-multipart message holding part 1, its own entity. A section carries its entity, its span (`BODY[n]` is its body, `BODY[n.MIME]` its header) and, for the root and a parsed `message/rfc822` or `message/global` part, the span of the message held (`BODY[n.HEADER]`, `BODY[n.TEXT]`). `MimeHeaderBlock::len` is the block's written length. Sizes are range lengths; line counts are the line feeds of a body slice, left to the caller.

Code: src/tree/span.rs (new), `MimeHeaderBlock::len` in src/tree/header.rs. The parse and the tree are untouched: no field, no allocation and no work were added to the parse loop; the spans cost one walk and one vector when asked. Nothing was timed.

Tests (tests/span.rs): on every vendored fixture in its three forms, each span slices its entity's chunks out of the input and equals the offsets tests/common/mod.rs holds to mail-parser's frozen answers; on the 336 of 462 inputs parsed with no anomaly (45 with a nested message), the spans equal a naive split on the boundary lines and the first empty line; a hand-written message nesting a `multipart/alternative`, a `message/rfc822` holding a multipart and one holding a single part is numbered and sliced as IMAP names it; a single-part message has part 1; spans index the serialized bytes after an edit.

Left out: an encoded `message/rfc822` (base64 or quoted-printable, which RFC 2046 5.2.1 forbids) stays a leaf with no nested sections, its decoded tree (`MimeEntity::nested`) indexing other bytes; an unparsed body holds no section until expanded.

Capabilities moved: parsing.
