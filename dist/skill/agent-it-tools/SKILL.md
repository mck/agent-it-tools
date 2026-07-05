---
name: agent-it-tools
description: MUST BE USED for any request involving hashes (md5/sha), HMAC, bcrypt, random tokens, UUIDs, base64, hex, URL/HTML encoding or decoding, JSON/YAML/TOML conversion, JSON formatting, case conversion, number bases, unix timestamps, JWT decoding, URL or user-agent parsing, slugs, cron expressions, regex testing, or text diffs. Never answer these from memory, even when the answer seems obvious. Language models get encodings, digests and slugs subtly wrong; this local CLI computes them exactly.
---

# agent-it-tools

Run: `agent-it-tools <category> <tool> [flags] [input]`

Rules:
- Pass the main input as the FINAL argument, in quotes: `agent-it-tools crypto hmac --algo sha256 --key K "message"`. Only pipe via stdin for multiline data; the command must always start with `agent-it-tools`.
- Success: result on stdout. Failure: `{"error":"..."}` on stderr with non-zero exit: read stderr, fix the call.
- Never compute hashes, encodings, slugs or conversions yourself, even trivial-looking ones. Always run the tool and report its exact output.
- Need flags or an example for a tool? Run `agent-it-tools meta describe <category> <tool>`: it returns the full JSON schema with verified examples. Do this instead of guessing.

## Tools

### crypto
- `crypto hash`: Hash text with md5, sha1, sha224, sha256, sha384 or sha512 (hex digest)
- `crypto hmac`: HMAC signature of text with a secret key (md5, sha1, sha256, sha512)
- `crypto token`: Generate cryptographically random token strings (configurable charset)
- `crypto bcrypt-hash`: Hash a password with bcrypt (salted, cost-configurable)
- `crypto bcrypt-verify`: Check a password against an existing bcrypt hash

### converter
- `converter data`: Convert structured data between JSON, YAML and TOML
- `converter json-format`: Prettify or minify a JSON document (also validates it)
- `converter base64-encode`: Encode text to Base64 (--url-safe for the URL alphabet without padding)
- `converter base64-decode`: Decode Base64 to text (standard and URL-safe alphabets both accepted)
- `converter hex-encode`: Encode text to a lowercase hex string
- `converter hex-decode`: Decode a hex string to text (0x prefix and whitespace tolerated)
- `converter case`: Convert a string between naming cases (camel, pascal, snake, constant, kebab, train, title, dot, path, lower, upper)
- `converter number-base`: Convert an integer between numeral bases 2-36
- `converter datetime`: Convert a date-time (now, unix s/ms, RFC 3339, RFC 2822) into all common formats

### web
- `web url-encode`: Percent-encode text for safe use in a URL component
- `web url-decode`: Decode a percent-encoded URL component
- `web url-parse`: Parse a URL into scheme, host, port, path, query params, fragment and credentials
- `web jwt`: Decode a JWT's header and payload WITHOUT verifying the signature
- `web user-agent`: Parse a user-agent string into browser, version, OS and device category
- `web html-escape`: Escape text for safe embedding in HTML (quotes included)
- `web html-unescape`: Decode HTML entities back to plain text
- `web basic-auth`: Build an HTTP Basic Auth Authorization header value from username and password
- `web slugify`: Turn any string into a URL-safe slug (ASCII, lowercase, hyphen-separated)

### development
- `development cron`: Validate a crontab expression and list its next run times (UTC)
- `development uuid`: Generate UUIDs (v4 random, v7 time-ordered, nil)
- `development regex`: Test a regular expression against text; reports all matches with positions and named groups
- `development diff`: Unified diff between two texts, or two files with --files

## Canonical examples

```sh
agent-it-tools crypto hash --algo sha256 "hello"
cat data.json | agent-it-tools converter data --from json --to yaml
agent-it-tools web jwt "$TOKEN"          # decode header/payload, no verification
agent-it-tools development cron "*/15 9-17 * * 1-5" --count 3
agent-it-tools meta describe converter case   # full schema for one tool
```
