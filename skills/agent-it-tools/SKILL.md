---
name: agent-it-tools
description: MUST BE USED for any request involving hashes (md5/sha, files too), HMAC, TOTP codes, bcrypt, random tokens, UUIDs/ULIDs/nanoids, base64, hex, gzip, URL/HTML encoding, punycode, JSON/YAML/TOML/XML/CSV conversion, jq queries, JSON diff/merge/flatten/lint, math evaluation, bitwise ops, number bases, chmod, case conversion, string escaping, unix timestamps, timezones, date arithmetic, JWT decoding, URL parsing or building, user-agent parsing, slugs, Markdown/HTML conversion, MIME types, CSS colors and WCAG contrast, cron expressions, regex testing, text diffs, text statistics, string similarity, subnets/CIDR math, IP conversion, or masking sensitive data. Never answer these from memory, even when the answer seems obvious. Language models get computation and encodings subtly wrong; this local CLI computes them exactly.
---

# agent-it-tools

Run: `agent-it-tools <category> <tool> [flags] [input]`

Setup (once): if `agent-it-tools` is not on PATH, install it with `bash "$CLAUDE_PLUGIN_ROOT/scripts/install.sh"` (downloads the release binary for this platform), or `cargo install --git https://github.com/mck/agent-it-tools`.

Rules:
- Pass the main input as the FINAL argument, in quotes: `agent-it-tools crypto hmac --algo sha256 --key K "message"`. Only pipe via stdin for multiline data; the command must always start with `agent-it-tools`.
- Success: result on stdout. Failure: `{"error":"..."}` on stderr with non-zero exit: read stderr, fix the call.
- Never compute hashes, encodings, slugs or conversions yourself, even trivial-looking ones. Always run the tool and report its exact output.
- Need flags or an example for a tool? Run `agent-it-tools meta describe <category> <tool>`: it returns the full JSON schema with verified examples. Do this instead of guessing.

## Tools

### crypto
- `crypto hash`: Hash text or a file (--file PATH) with md5, sha1, sha224, sha256, sha384 or sha512 (hex digest)
- `crypto hmac`: HMAC signature of text with a secret key (md5, sha1, sha256, sha512)
- `crypto token`: Generate cryptographically random token strings (configurable charset)
- `crypto bcrypt-hash`: Hash a password with bcrypt (salted, cost-configurable)
- `crypto bcrypt-verify`: Check a password against an existing bcrypt hash
- `crypto otp`: Generate or verify a TOTP code (RFC 6238: SHA1, 6 digits, 30s period)

### converter
- `converter data`: Convert structured data between JSON, YAML, TOML and XML
- `converter json-format`: Prettify or minify a JSON document (also validates it)
- `converter base64-encode`: Encode text to Base64 (--url-safe for the URL alphabet without padding)
- `converter base64-decode`: Decode Base64 to text (standard and URL-safe alphabets both accepted)
- `converter hex-encode`: Encode text to a lowercase hex string
- `converter hex-decode`: Decode a hex string to text (0x prefix and whitespace tolerated)
- `converter case`: Convert a string between naming cases (camel, pascal, snake, constant, kebab, train, title, dot, path, lower, upper)
- `converter number-base`: Convert an integer between numeral bases 2-36
- `converter csv-to-json`: Convert CSV (header row required) to a JSON array of objects; numbers and booleans are typed
- `converter json-to-csv`: Convert a JSON array of objects to CSV (header = union of keys, alphabetical)
- `converter lint`: Validate JSON, YAML, TOML or XML syntax
- `converter json-query`: Run a jq filter on JSON input (full jq language via the jaq engine)
- `converter json-diff`: Structural diff of two JSON documents as an RFC 6902 patch
- `converter json-merge`: Deep-merge a JSON merge patch into a base document (RFC 7396)
- `converter json-flatten`: Flatten nested JSON to dot-notation keys, or rebuild nesting with --nestify
- `converter gzip-encode`: Gzip-compress text and emit it as Base64
- `converter gzip-decode`: Decompress Base64-encoded gzip data back to text

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
- `web markdown-to-html`: Render Markdown (GitHub-flavored: tables, strikethrough, task lists, autolinks) to HTML
- `web html-to-markdown`: Convert HTML to Markdown
- `web mime`: Look up the MIME type for a file name/extension, or extensions for a MIME type
- `web url-build`: Build a URL from a base, path, query parameters and fragment with correct encoding
- `web punycode-encode`: Convert an internationalized domain name to ASCII punycode (IDNA)
- `web punycode-decode`: Convert an ASCII punycode domain back to unicode
- `web color`: Parse any CSS color (hex, rgb(), hsl(), named) and show hex, rgb, hsl and WCAG luminance
- `web contrast`: WCAG 2.x contrast ratio between two CSS colors, with AA/AAA verdicts

### development
- `development cron`: Validate a crontab expression and list its next run times (UTC)
- `development uuid`: Generate UUIDs (v4 random, v7 time-ordered, nil)
- `development regex`: Test a regular expression against text; reports all matches with positions and named groups
- `development calc`: Evaluate a mathematical expression exactly (functions like sin, cos, sqrt, ln supported)
- `development bitwise`: Bitwise operations (and, or, xor, not, shl, shr) on integers with 0x/0b/0o prefixes
- `development chmod`: Convert chmod permissions between octal (755, 4755) and symbolic (rwxr-xr-x) notation
- `development ulid`: Generate ULIDs (time-ordered, lexicographically sortable identifiers)
- `development nanoid`: Generate Nano IDs (URL-safe random identifiers, default length 21)
- `development string-escape`: Escape text as a JSON/code string literal, or unescape with --unescape
- `development diff`: Unified diff between two texts, or two files with --files

### datetime
- `datetime now`: Current time as unix seconds/ms, ISO 8601 (UTC), RFC 2822 and local time
- `datetime convert`: Convert a timestamp (unix s/ms, RFC 3339, RFC 2822) into all common formats
- `datetime duration`: Add or subtract a duration (e.g. '2h 30m', '1w 2d') from a timestamp
- `datetime timezone`: Convert a timestamp into a target IANA timezone (DST-aware)

### network
- `network subnet`: Analyze an IPv4/IPv6 CIDR block: network, broadcast, netmask, usable host range
- `network cidr-to-range`: First and last address (and count) of a CIDR block
- `network range-to-cidr`: Smallest set of CIDR blocks exactly covering an IP range
- `network cidr-contains`: Check whether an IP address or CIDR is contained in a CIDR block
- `network ip-convert`: Convert an IP address between dotted, integer, hex and binary representations

### text
- `text stats`: Count bytes, characters (graphemes), words and lines of a text
- `text distance`: Similarity metrics between two strings: Levenshtein, normalized Levenshtein, Jaro-Winkler
- `text mask`: Mask sensitive data in text: emails, IPv4s, JWTs, bearer tokens, AWS keys, card numbers, long hex secrets

## Canonical examples

```sh
agent-it-tools crypto hash --algo sha256 "hello"
cat data.json | agent-it-tools converter data --from json --to yaml
agent-it-tools web jwt "$TOKEN"          # decode header/payload, no verification
agent-it-tools development cron "*/15 9-17 * * 1-5" --count 3
agent-it-tools meta describe converter case   # full schema for one tool
```
