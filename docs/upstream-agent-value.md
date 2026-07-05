# Upstream agent-value map

All 455 tools in sharevb/it-tools rated 0-10 for LLM-agent value.
Full data: [upstream-agent-value.json](upstream-agent-value.json).

| tier | tools |
|---|---|
| 9-10 | 35 |
| 7-8 | 162 |
| 5-6 | 159 |
| 3-4 | 49 |
| 0-2 | 50 |

## Every tool rated 8+

| rating | id | status | crates | description |
|---|---|---|---|---|
| 10 | `base64-string-converter` | implemented |  | Simply encode and decode strings into their base64 representation. |
| 10 | `hash-text` | implemented |  | Hash a text string using the function you need : MD5, SHA1, SHA256, SHA224, SHA5 |
| 10 | `hmac-generator` | implemented |  | Computes a hash-based message authentication code (HMAC) using a secret key and  |
| 10 | `math-evaluator` | planned | evalexpr | A calculator for evaluating mathematical expressions. You can use functions like |
| 9 | `bcrypt` | implemented |  | Hash and compare text string using bcrypt. Bcrypt is a password-hashing function |
| 9 | `binary-calculator` | planned | std | Calculate bitwise/binary operations (AND, OR, XOR, NOT, shifts) between two numb |
| 9 | `cidr-in-cidr` | planned | ipnet | Given a CIDR/IP Range/Wildcard IP/IP Mask, tell if a given IPv4-6/Range/CIDR/Wil |
| 9 | `crontab-generator` | implemented |  | Validate and generate crontab and get the human-readable description of the cron |
| 9 | `csv-to-json` | planned | csv | Convert CSV to JSON with automatic header detection. |
| 9 | `date-duration-calculator` | planned | chrono | Add/substract durations from a specific date |
| 9 | `date-time-converter` | implemented |  |  |
| 9 | `file-hasher` | planned | sha2 + std::fs | Compute Hash of files |
| 9 | `html-entities` | implemented | html-escape | Escape or unescape HTML entities (replace characters like <,>, &, " and \' with  |
| 9 | `integer-base-converter` | implemented | std |  |
| 9 | `ip-cidr-to-range` | planned | ipnet | Calculate IP Range from a CIDR (IPv4/6) |
| 9 | `ip-range-to-cidr` | planned | ipnet | Calculate CIDR(s) from an IP Range (IPv4/6) |
| 9 | `ipv4-subnet-calculator` | planned | ipnet | Parse your IPv4 CIDR blocks and get all the info you need about your subnet. |
| 9 | `ipv6-subnet-calculator` | planned | ipnet | Parse your IPv6 CIDR blocks and get all the info you need about your sub network |
| 9 | `jq-tester` | planned | jaq-core | Test jq/JSONPath expression against a JSON content |
| 9 | `json-diff` | planned | serde_json + custom walk | Compare two JSON objects and get the differences between them. |
| 9 | `json-query` | planned | jaq-core / serde_json | Run JSON Query lang on a given JSON content |
| 9 | `json-to-csv` | planned | csv | Convert JSON to CSV with automatic header detection. |
| 9 | `json-to-toml` | implemented |  | Parse and convert JSON to TOML. |
| 9 | `json-to-yaml-converter` | implemented |  | Simply convert JSON to YAML with this live online converter. |
| 9 | `jwt-parser` | implemented |  | Parse and decode your JSON Web Token (jwt) and display its content. |
| 9 | `regex-tester` | implemented |  | Test your regular expressions with sample text. |
| 9 | `text-diff` | implemented | similar | Compare two texts and see the differences between them. |
| 9 | `timezone-converter` | planned | chrono-tz | Convert Date-Time from a timezone to others and get timezone vs countries infos |
| 9 | `token-generator` | implemented |  | Generate random string with the chars you want, uppercase or lowercase letters,  |
| 9 | `toml-to-json` | implemented |  | Parse and convert TOML to JSON. |
| 9 | `url-encoder` | implemented |  | Encode to url-encoded format (also known as "percent-encoded") or decode from it |
| 9 | `url-parser` | implemented |  | Parse a URL into its separate constituent parts (protocol, origin, params, port, |
| 9 | `uuid-generator` | implemented |  | A Universally Unique Identifier (UUID) is a 128-bit number used to identify info |
| 9 | `xml-to-json` | planned | quick-xml (serde) | Convert XML to JSON |
| 9 | `yaml-to-json-converter` | implemented |  | Simply convert YAML to JSON with this online live converter. |
| 8 | `ansible-vault-crypt-decrypt` | planned |  | Encrypt and decrypt Ansible Vault Secrets |
| 8 | `argon2-hash` | planned | argon2 | Compute Argon2 hashes with parameters |
| 8 | `base64-hex-converter` | planned |  | Simply encode and decode Hex array into a their base64 representation. |
| 8 | `basic-auth-generator` | implemented |  | Generate a base64 basic auth header from a username and password. |
| 8 | `case-converter` | implemented |  | Transform the case of a string and choose between different formats |
| 8 | `certificate-key-parser` | planned | x509-parser | Parse Key and Certificate |
| 8 | `chmod-calculator` | planned | std (octal math) | Compute your chmod permissions and commands with this online chmod calculator. |
| 8 | `color-contrast-checker` | planned | palette | Check the WCAG contrast level between two colors |
| 8 | `color-converter` | planned | csscolorparser / palette | Convert color between the different formats (hex, rgb, hsl and css name) |
| 8 | `crc-calculator` | planned | crc | Compute text or file CRC (CRC1, CRC8, CRC8 1-Wire, CRC8 DVB-S2, CRC16, CRC16 CCI |
| 8 | `csv-to-data` | planned |  | Convert CSV file to JSON, YAML, CSV, SQL INSERT, XML, Markdown or XLSX |
| 8 | `days-calculator` | planned |  | Calculate days interval, holidays, difference, business times |
| 8 | `docker-compose-to-docker-run-converter` | planned |  | Turns Docker Compose filt to docker run command(s)! |
| 8 | `docker-compose-validator` | planned |  | Validate Docker Compose files against CommonSpec schema |
| 8 | `docker-inspect-to-docker-run` | planned |  | Convert docker inspect command json result back to Docker run command |
| 8 | `docker-run-to-docker-compose-converter` | planned |  | Transforms "docker run" commands into docker-compose files! |
| 8 | `duration-calculator` | planned |  | Calculate/parse durations |
| 8 | `ecdsa-key-pair-generator` | planned |  | Generate new random ECDSA private and public keys (with or without passphrase). |
| 8 | `ed25519-key-pair-generator` | planned |  | Generate new random Ed25519 private and public keys (with or without passphrase) |
| 8 | `email-parser` | planned |  | Parse and extract information from raw Email content |
| 8 | `encryption` | planned | aes-gcm / cbc | Encrypt clear text and decrypt ciphertext using crypto algorithms like AES, Trip |
| 8 | `file-type` | planned | infer | Identify the type of a file |
| 8 | `floating-point-number-converter` | planned | std |  |
| 8 | `gpt-token-encoder` | planned |  | Encode text to GPT tokens and decode GPT tokens back to text |
| 8 | `gpt-token-estimator` | planned | tiktoken-rs | OpenAI GPT Token Estimator |
| 8 | `gzip-converter` | planned | flate2 | Convert text from/to gzip/deflate |
| 8 | `hex-converter` | implemented |  | Encode and decode Hex buffers to number (bits, endianess, sign or floating point |
| 8 | `html-to-markdown` | planned | htmd | Convert HTML (either from clipboard) to Markdown |
| 8 | `iban-validator-and-parser` | planned | iban_validate | Validate and parse IBAN numbers. Check if an IBAN is valid and get the country,  |
| 8 | `integers-to-ip` | planned | std::net | Convert integers to IP |
| 8 | `ip-include-exclude` | planned |  | Substract a disallowed IP Ranges/Mask/CIDR list from an allowed IP Ranges/Mask/C |
| 8 | `ipv4-address-converter` | planned | std::net | Convert an IP address into decimal, binary, hexadecimal, or even an IPv6 represe |
| 8 | `ipv4-range-expander` | planned |  | Given a start and an end IPv4 address, this tool calculates a valid IPv4 subnet  |
| 8 | `ipv6-address-converter` | planned | std::net | Convert an ip address into decimal, binary, hexadecimal and get infos |
| 8 | `json-escaper` | planned |  | Escape and unescape JSON string |
| 8 | `json-flatten-nestify` | planned | serde_json | Flatten or nestify/unflatten JSON content (ie, {'{'}a:{'{'}b:1{'}'}{'}'} vs {'{' |
| 8 | `json-linter` | planned | serde_json | Check and lint JSON content |
| 8 | `json-merger` | planned | serde_json | Merge deeply two JSON content |
| 8 | `json-minify` | implemented |  | Minify and compress your JSON by removing unnecessary whitespace. |
| 8 | `json-to-schema` | planned | schemars-style custom | Convert JSON data to JSON Schema, MySQL DDL, Mongoose Schema, Google BigQuery sc |
| 8 | `json-to-xml` | planned | quick-xml | Convert JSON to XML |
| 8 | `json-viewer` | implemented |  |  |
| 8 | `luhn-validator` | planned | luhn | Check and generate key for identifier validated by a Luhn checknum |
| 8 | `markdown-to-html` | planned | comrak | Convert Markdown to HTML and allow to print (as PDF) |
| 8 | `mongo-objectid-converter` | planned | bson | Convert between MongoDB ObjectId and internal timestamp |
| 8 | `msgpack-to-json` | planned | rmp-serde | Convert MessagePack file to JSON |
| 8 | `nanoid-generator` | planned | nanoid | Generate random, unique, and URL-friendly IDs for your applications. |
| 8 | `otp-code-generator-and-validator` | planned | totp-rs |  |
| 8 | `parquets-reader` | planned |  | Read parquet file as JSON object arrays |
| 8 | `passphrase-generator` | planned |  | Generate random memoizable Passphrases |
| 8 | `pdf-text-extractor` | planned | pdf-extract | Extract text from PDF |
| 8 | `phone-parser-and-formatter` | planned | phonenumber | Parse, validate and format phone numbers. Get information about the phone number |
| 8 | `punycode-converter` | planned | idna | Convert international unicode domain names or emails from/to ASCII Punycode vers |
| 8 | `random-numbers-generator` | planned |  | Generate random numbers (decimal, hexadecimal). With denied characters, you can  |
| 8 | `rsa-ecdsa-signing` | planned |  | Sign data and verify signature using RSA/DSA/ECDSA Keys |
| 8 | `rsa-key-pair-generator` | planned |  | Generate new random RSA private and public keys (with or without passphrase). |
| 8 | `sensitive-data-masker` | planned | regex | Clean sensitive data from textual content (ie logs) |
| 8 | `slugify-string` | implemented |  | Make a string url, filename and id safe. |
| 8 | `snowflake-id-extractor` | planned | std (bit math) | Extract timestamp, machine ID, and sequence number from a Snowflake ID |
| 8 | `ssl-cert-converter` | planned |  | Convert SSL Certificate from different formats |
| 8 | `string-escaper` | planned |  | Escape string to code language version |
| 8 | `text-statistics` | planned | unicode-segmentation | Get information about a text, the number of characters, the number of words, its |
| 8 | `toml-linter` | planned | toml | Lint and check TOML content |
| 8 | `toml-to-yaml` | implemented |  | Parse and convert TOML to YAML. |
| 8 | `toon-to-json` | planned |  | Convert TOON representation to JSON object for LLM usage |
| 8 | `ulid-generator` | planned | ulid | Generate random Universally Unique Lexicographically Sortable Identifier (ULID). |
| 8 | `user-agent-parser` | implemented |  | Detect and parse Browser, Engine, OS, CPU, and Device type/model from an user-ag |
| 8 | `xml-linter` | planned | quick-xml | Lint XML content for syntax error |
| 8 | `xpath-tester` | planned | sxd-xpath | Test XPath expression against XML content |
| 8 | `yaml-flatten-nestify` | planned |  | Flatten or nestify/unflatten YAML content (ie, a.b: 1 vs a: b: 1) |
| 8 | `yaml-merger` | planned | serde_yaml | Merge deeply two YAML content |
| 8 | `yaml-to-toml` | implemented |  | Parse and convert YAML to TOML. |

## Notes

- 41 tools fell back to category-default ratings (all in the 5-6 band; every 0-4 and 7+ rating is explicit).
- (net) = needs internet/external service at runtime.
- file_io flags (binary/file-based tools) are in the JSON.
## Recommended wave 1 (~25 CLI tools, ~40 upstream ids)

High rating, low implementation effort, mature crates. One CLI tool often
covers several upstream ids.

| ours | upstream ids | crates |
|---|---|---|
| `converter data --from/--to xml` | json-to-xml, xml-to-json | quick-xml |
| `converter csv` | csv-to-json, json-to-csv, csv-to-data (partial) | csv |
| `converter lint` | json-linter, toml-linter, xml-linter | serde stack |
| `converter json-query` | json-query, jq-tester, jsonpath-memo | jaq-core |
| `converter json-diff` | json-diff | serde_json |
| `converter json-merge` | json-merger, yaml-merger | serde_json |
| `converter json-flatten` | json-flatten-nestify, yaml-flatten-nestify | serde_json |
| `converter gzip` | gzip-converter | flate2 |
| `crypto hash --file` | file-hasher | sha2 |
| `crypto otp` | otp-code-generator-and-validator | totp-rs |
| `web markdown` | markdown-to-html, html-to-markdown, markdown-to-text | comrak, htmd |
| `web mime` | mime-types | mime_guess |
| `web url-build` | url-builder | url |
| `web punycode` | punycode-converter | idna |
| `web color` | color-converter, color-contrast-checker | csscolorparser, palette |
| `development calc` | math-evaluator | evalexpr |
| `development bitwise` | binary-calculator | std |
| `development chmod` | chmod-calculator | std |
| `development ulid` / `nanoid` | ulid-generator, nanoid-generator | ulid, nanoid |
| `development string-escape` | string-escaper, json-escaper | std |
| `datetime duration` | date-duration-calculator, duration-calculator | chrono |
| `datetime timezone` | timezone-converter | chrono-tz |
| `network subnet` (new category) | ipv4/ipv6-subnet-calculator | ipnet |
| `network cidr` | ip-cidr-to-range, ip-range-to-cidr, cidr-in-cidr | ipnet |
| `network ip-convert` | ipv4/6-address-converter, integers-to-ip | std::net |
| `text stats` (new category) | text-statistics | unicode-segmentation |
| `text mask` | sensitive-data-masker | regex |
| `text distance` | levenshtein-calculator | strsim |
