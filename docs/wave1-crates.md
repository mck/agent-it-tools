# Wave-1 crate research

Verified against crates.io on 2026-07-05 (version, last release, all-time /
recent-90d downloads, license). Verdict legend: USE (primary pick), ALT
(viable fallback), AVOID (license or abandonment).

## License landmines found

- `evalexpr` (math-evaluator candidate) is **AGPL-3.0-only** since v12: viral
  copyleft, incompatible with our MIT binary. AVOID. Use `exmex`
  (MIT OR Apache-2.0, active 2026-05) instead.
- `html2md` is **GPL-3.0+**. AVOID. Use `htmd` (Apache-2.0, active).
- `xmltojson` is **LGPL-3.0+** and tiny. AVOID. Talk to `quick-xml` directly.

## Verdicts by wave-1 tool

| tool | verdict | crate | version | updated | recent dl | license |
|---|---|---|---|---|---|---|
| data --from/--to xml | USE | quick-xml | 0.41.0 | 2026-06 | 77.1M | MIT |
| | AVOID | xmltojson | 0.3.0 | 2025-07 | 2.7k | LGPL-3.0+ |
| csv | USE | csv | 1.4.0 | 2025-10 | 37.9M | Unlicense/MIT |
| json-query (jq) | USE | jaq-core + jaq-json + jaq-std | 3.1.0 / 2.0.1 / 3.0.1 | 2026-06 | 0.9M | MIT |
| | note | jaq-interpret is the LEGACY 1.x API, do not use | | 2024-06 | | |
| json-query (JSONPath, later) | ALT | jsonpath-rust | 1.0.4 | 2025-07 | 12.9M | MIT |
| json-diff | USE | json-patch (RFC 6902 diff) | 4.2.0 | 2026-04 | 19.2M | MIT/Apache-2.0 |
| | AVOID | serde-json-diff | 0.2.0 | 2023-08 | 27k | MIT (stale) |
| json-merge / flatten / lint | USE | serde stack (already in tree) | | | | |
| gzip | USE | flate2 | 1.1.9 | 2026-02 | 116.5M | MIT OR Apache-2.0 |
| crypto otp | USE | totp-rs | 5.7.2 | 2026-06 | 2.8M | MIT |
| | AVOID | otpauth | 0.5.1 | 2024-08 | 5.6k | MIT (stale) |
| markdown-to-html | USE | comrak (full GFM) | 0.53.0 | 2026-07 | 1.8M | BSD-2-Clause |
| | ALT | pulldown-cmark (lighter, less GFM) | 0.13.4 | 2026-05 | 33.5M | MIT |
| html-to-markdown | USE | htmd | 0.5.4 | 2026-04 | 1.1M | Apache-2.0 |
| | AVOID | html2md | 0.2.15 | 2025-01 | 200k | GPL-3.0+ |
| web mime | USE | mime_guess | 2.0.5 | 2024-06 | 45.1M | MIT |
| web punycode | USE | idna (already a transitive dep via url) | 1.1.0 | 2025-08 | 161.2M | MIT OR Apache-2.0 |
| web color | USE | csscolorparser | 0.8.3 | 2026-03 | 7.2M | MIT OR Apache-2.0 |
| | ALT | palette (only if we need color math beyond parsing) | 0.7.6 | 2024-04 | 3.0M | MIT OR Apache-2.0 |
| development calc | USE | exmex | 0.21.0 | 2026-05 | 4.9k | MIT OR Apache-2.0 |
| | AVOID | evalexpr | 13.1.0 | 2025-11 | 1.4M | AGPL-3.0-only |
| | AVOID | fasteval / meval (dead since 2020 / 2018) | | | | |
| development ulid | USE | ulid | 1.2.1 | 2025-03 | 8.2M | MIT |
| development nanoid | USE | nanoid | 0.5.0 | 2026-04 | 4.5M | MIT |
| datetime timezone | USE | chrono-tz | 0.10.4 | 2025-07 | 27.7M | MIT OR Apache-2.0 |
| network subnet/cidr | USE | ipnet | 2.12.0 | 2026-03 | 107.7M | MIT OR Apache-2.0 |
| | ALT | ipnetwork / cidr | | | | |
| text stats | USE | unicode-segmentation | 1.13.3 | 2026-06 | 101.2M | MIT OR Apache-2.0 |
| text distance | USE | strsim | 0.11.1 | 2024-04 | 180.7M | MIT |
| | AVOID | levenshtein (dead 2021; strsim covers it) | | | | |
| msgpack (wave 2) | USE | rmp-serde | 1.3.1 | 2025-12 | 21.6M | MIT |

## Notes

- `exmex` is small (4.9k recent downloads) but actively maintained and the
  only permissively-licensed maintained expression evaluator; wrap it behind
  our own `development calc` interface so it stays swappable.
- WCAG contrast for `web color` is ~20 lines of relative-luminance math on
  top of csscolorparser; no extra crate needed.
- `bitwise`, `chmod`, `string-escape`, `ip-convert`, flatten/merge/lint need
  no new crates at all (std + serde stack already in the tree).
