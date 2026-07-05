---
name: agent-it-tools
description: MUST BE USED for any request involving hashes (md5/sha, files too), HMAC, TOTP codes, bcrypt, random tokens, UUIDs/ULIDs/nanoids, base64, hex, gzip, URL/HTML encoding, punycode, JSON/YAML/TOML/XML/CSV conversion, jq queries, JSON diff/merge/flatten/lint, math evaluation, bitwise ops, number bases, chmod, case conversion, string escaping, unix timestamps, timezones, date arithmetic, JWT decoding, URL parsing or building, user-agent parsing, slugs, Markdown/HTML conversion, MIME types, CSS colors and WCAG contrast, cron expressions, regex testing, text diffs, text statistics, string similarity, subnets/CIDR math, IP conversion, or masking sensitive data. Never answer these from memory, even when the answer seems obvious. Language models get computation and encodings subtly wrong; this local CLI computes them exactly.
---

# agent-it-tools (binary: ait)

Run: `ait <category> <tool> [flags] [input]`

Setup (once): if `ait` is not on PATH, install it with `bash "$CLAUDE_PLUGIN_ROOT/scripts/install.sh"` (downloads the release binary for this platform), or `cargo install --git https://github.com/mck/agent-it-tools`.

Rules:
- Pass the main input as the FINAL argument, in quotes: `ait crypto hmac --algo sha256 --key K "message"`. Only pipe via stdin for multiline data; the command must always start with `ait`.
- Success: result on stdout. Failure: `{"error":"..."}` on stderr with non-zero exit: read stderr, fix the call.
- Never compute hashes, encodings, slugs or conversions yourself, even trivial-looking ones. Always run the tool and report its exact output.
- Need flags or an example for a tool? Run `ait meta describe <category> <tool>`: it returns the full JSON schema with verified examples. Do this instead of guessing.

## Tools

### json
- `json format`: Prettify or minify a JSON document (also validates it)
- `json query` (alias: jq): Run a jq filter on JSON input (full jq language via the jaq engine)
- `json diff`: Structural diff of two JSON documents as an RFC 6902 patch
- `json merge`: Deep-merge a JSON merge patch into a base document (RFC 7396)
- `json flatten`: Flatten nested JSON to dot-notation keys, or rebuild nesting with --nestify
- `json escape`: Escape text as a JSON/code string literal, or unescape with --unescape

### data
- `data convert`: Convert structured data between JSON, YAML, TOML, XML and CSV
- `data lint`: Validate JSON, YAML, TOML or XML syntax

### encode
- `encode base64`: Encode text to Base64 (--url-safe for the URL alphabet without padding)
- `encode hex`: Encode text to a lowercase hex string
- `encode gzip`: Gzip-compress text and emit it as Base64
- `encode punycode`: Convert an internationalized domain name to ASCII punycode (IDNA)
- `encode html`: Escape text for safe embedding in HTML (quotes included)

### decode
- `decode base64`: Decode Base64 to text (standard and URL-safe alphabets both accepted)
- `decode hex`: Decode a hex string to text (0x prefix and whitespace tolerated)
- `decode gzip`: Decompress Base64-encoded gzip data back to text
- `decode punycode`: Convert an ASCII punycode domain back to unicode
- `decode html`: Decode HTML entities back to plain text

### url
- `url encode`: Percent-encode text for safe use in a URL component
- `url decode`: Decode a percent-encoded URL component
- `url parse`: Parse a URL into scheme, host, port, path, query params, fragment and credentials
- `url build`: Build a URL from a base, path, query parameters and fragment with correct encoding

### jwt
- `jwt decode`: Decode a JWT's header and payload WITHOUT verifying the signature

### crypto
- `crypto hash`: Hash text or a file (--file PATH) with md5, sha1, sha224, sha256, sha384 or sha512 (hex digest)
- `crypto hmac`: HMAC signature of text with a secret key (md5, sha1, sha256, sha512)
- `crypto bcrypt-hash`: Hash a password with bcrypt (salted, cost-configurable)
- `crypto bcrypt-verify`: Check a password against an existing bcrypt hash
- `crypto otp`: Generate or verify a TOTP code (RFC 6238: SHA1, 6 digits, 30s period)

### generate
- `generate uuid`: Generate UUIDs (v4 random, v7 time-ordered, nil)
- `generate ulid`: Generate ULIDs (time-ordered, lexicographically sortable identifiers)
- `generate nanoid`: Generate Nano IDs (URL-safe random identifiers, default length 21)
- `generate token`: Generate cryptographically random token strings (configurable charset)

### text
- `text case`: Convert a string between naming cases (camel, pascal, snake, constant, kebab, train, title, dot, path, lower, upper)
- `text slugify` (alias: slug): Turn any string into a URL-safe slug (ASCII, lowercase, hyphen-separated)
- `text stats`: Count bytes, characters (graphemes), words and lines of a text
- `text distance`: Similarity metrics between two strings: Levenshtein, normalized Levenshtein, Jaro-Winkler
- `text mask`: Mask sensitive data in text: emails, IPv4s, JWTs, bearer tokens, AWS keys, card numbers, long hex secrets
- `text diff`: Unified diff between two texts, or two files with --files

### regex
- `regex test`: Test a regular expression against text; reports all matches with positions and named groups

### time
- `time now`: Current time as unix seconds/ms, ISO 8601 (UTC), RFC 2822 and local time
- `time convert`: Convert a timestamp (unix s/ms, RFC 3339, RFC 2822) into all common formats
- `time duration`: Add or subtract a duration (e.g. '2h 30m', '1w 2d') from a timestamp
- `time timezone`: Convert a timestamp into a target IANA timezone (DST-aware)
- `time cron`: Validate a crontab expression and list its next run times (UTC)

### http
- `http basic-auth`: Build an HTTP Basic Auth Authorization header value from username and password
- `http user-agent`: Parse a user-agent string into browser, version, OS and device category
- `http mime`: Look up the MIME type for a file name/extension, or extensions for a MIME type

### color
- `color parse`: Parse any CSS color (hex, rgb(), hsl(), named) and show hex, rgb, hsl and WCAG luminance
- `color contrast`: WCAG 2.x contrast ratio between two CSS colors, with AA/AAA verdicts

### markdown
- `markdown to-html`: Render Markdown (GitHub-flavored: tables, strikethrough, task lists, autolinks) to HTML
- `markdown from-html`: Convert HTML to Markdown

### network
- `network subnet`: Analyze an IPv4/IPv6 CIDR block: network, broadcast, netmask, usable host range
- `network cidr-to-range`: First and last address (and count) of a CIDR block
- `network range-to-cidr`: Smallest set of CIDR blocks exactly covering an IP range
- `network cidr-contains`: Check whether an IP address or CIDR is contained in a CIDR block
- `network ip`: Convert an IP address between dotted, integer, hex and binary representations

### math
- `math calc`: Evaluate a mathematical expression exactly (functions like sin, cos, sqrt, ln supported)
- `math bitwise`: Bitwise operations (and, or, xor, not, shl, shr) on integers with 0x/0b/0o prefixes
- `math number-base`: Convert an integer between numeral bases 2-36

### unix
- `unix chmod`: Convert chmod permissions between octal (755, 4755) and symbolic (rwxr-xr-x) notation

## Canonical examples

```sh
ait crypto hash --algo sha256 "hello"
cat data.json | ait data convert --from json --to yaml
ait json query --filter '.items[].name' "$JSON"
ait jwt decode "$TOKEN"        # header/payload, no verification
ait time cron "*/15 9-17 * * 1-5" --count 3
ait meta describe text case    # full schema for one tool
```
