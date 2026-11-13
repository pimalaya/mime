---
cairn: log
change: a-message-is-never-encoded
landed: 2026-11-12
---

# A message is never encoded

Landed [a-message-is-never-encoded](../changes/archive/a-message-is-never-encoded/proposal.md), found by Himalaya's live tests (an attached `.eml` and a forwarded message sent base64).

A `message/rfc822` or `message/global` leaf now has its line breaks written as the build's line ending, then takes the first identity encoding that carries it; when none does, its content is parsed and written as the nested message it holds, its leaves encoded. The boundary fast path counts such a leaf as written as is, which fixes a structure break when a message this builder produced was attached by its bytes (its `=_mime-rs_` boundary was reused by the outer multipart).

Code: `message_leaf` and `is_message` in src/build/entity.rs, `starts_with_field` in src/tree/parse.rs made crate-visible. Tests: three in tests/build.rs `review`, each failing on 0.1.0.

Capability moved: building (the transfer encoding and boundary requirements).
