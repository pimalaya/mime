---
cairn: delta
change: a-message-is-never-encoded
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: The transfer encoding is chosen per part

A `message/rfc822` or `message/global` leaf SHALL have its line breaks written as the build's line ending and SHALL be written under the first identity encoding that can carry it; when none can (8-bit content with `8bit` not allowed, a NUL, a line over 998 octets), its content SHALL be parsed and written as the nested message it holds, its own fields and leaves encoded by these rules, since RFC 2046 5.2.1 allows no other encoding. Such content opening with no field, or a build without the `parser` feature, SHALL be written as any other leaf. Binary content other than a message SHALL never be altered.

#### Scenario: A message file attached with LF line breaks
- GIVEN an ASCII message read from a file whose lines end in LF, attached as `message/rfc822` to a CRLF build
- WHEN the message is built
- THEN the attached message is written `7bit`, its lines ending in CRLF

#### Scenario: An 8-bit message attached when 8bit is not allowed
- GIVEN a message whose body is 8-bit UTF-8, attached as `message/rfc822` with `allow_8bit` off
- WHEN the message is built
- THEN the attached message is written as a nested message under no encoding, its body quoted-printable or base64, and reads back with the same subject and text

### Requirement: Boundaries never collide

The parts SHALL be written straight into the output, unchecked, only when no content written as is holds the generated prefix, a message leaf counting as written as is whatever it declares.

#### Scenario: A message this builder produced, attached as bytes
- GIVEN a multipart message this builder produced, attached by its bytes as a `message/rfc822` leaf
- WHEN the outer message is built
- THEN the outer boundary differs from the attached message's, and both parse back with their own parts

## REMOVED Requirements
