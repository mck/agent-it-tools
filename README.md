# agent-it-tools

**Language models predict. This binary computes.**

Ask a model for a sha256 digest, an HMAC signature, or a base64 decode and it
will answer fluently, whether or not the answer is real. Sometimes it is. And
for an agent pipeline, *sometimes* is just wrong: a fabricated digest is
byte-for-byte indistinguishable from a correct one to every downstream step.
Inconsistent capability is not a weaker form of capability. It is noise
wearing the costume of signal.

`agent-it-tools` (shipped as the `ait` binary) is a single static Rust binary that gives agents the
deterministic version of the [it-tools](https://github.com/sharevb/it-tools)
developer utility suite: hashing, HMAC, encodings, data-format conversion,
JWT/URL/user-agent parsing, cron, regex, diffs. Every answer is computed, the
same way, every time.

Measured on the smallest current Claude model (Haiku), same ten tasks,
identical prompts, three runs each, and a task only passes if ALL runs pass:

| | bare model | model + agent-it-tools |
|---|---|---|
| reliability score | 6/10 | **10/10** (30/30 runs) |
| sha256 / HMAC | 1/3 and 0/3, fabricated hex | 3/3, computed |
| base64 decode | 1/3, a coin-flip | 3/3 |
| cost per task | comparable | ~$0.014 |

<sub>`./evals/run.sh haiku both 3` on 2026-07-05, random non-memorizable
fixtures, graded against precomputed ground truth; reproduce it yourself with
the eval harness.</sub>

A capability that passes sometimes is not a weaker capability, it is noise:
the consuming pipeline cannot tell a lucky run from a fabricated one (see
[`evals/`](evals/README.md)). The tool arm's only failure mode is not invoking
the tool, and the shipped skill file exists to drive exactly that to zero.

## Install as a Claude Code plugin

The repository is itself a Claude Code plugin and marketplace:

```
/plugin marketplace add mck/agent-it-tools
/plugin install agent-it-tools@agent-it-tools
```

The bundled skill teaches the invocation contract and installs the release
binary on first use (`scripts/install.sh` picks the right platform build; or
`cargo install --git https://github.com/mck/agent-it-tools`). For other
runtimes: `dist/skill/` is a standalone skill, `dist/openai-tools.json` is an
OpenAI function-calling manifest, `dist/catalog.json` feeds anything else.

## Agent-first I/O contract

- **Success** → result written directly to `stdout` (plain text, or pretty
  JSON for structured tools). No spinners, colors, or ASCII art.
- **Failure** → `{"error": "reason"}` as valid JSON on `stderr`, non-zero exit
  code (`1` for errors; `crypto bcrypt-verify` exits `2` on a mismatch).
- Every tool takes its main body input as an **optional positional argument**;
  when omitted, it reads the full **stdin** pipe instead:

```sh
ait crypto hash --algo sha256 "hello"
cat payload.json | ait data convert --from json --to yaml
```

## Architecture: specs are the source of truth

Every tool ships with a hand-authored spec file under
[`specs/<category>/<tool>.toml`](specs/) carrying what the code cannot express:
a one-line summary written for an agent, when to use it, what the output means,
which upstream it-tools it covers, and **verified examples**. The specs are
embedded into the binary at compile time and merged with clap introspection
(flags, defaults, types) into a single *catalog*. Everything else derives from
that catalog:

```
specs/*.toml ──┐
               ├─► catalog ──► SKILL.md (Claude skill)
clap derive ───┘        ├────► Claude Code plugin
                        ├────► openai-tools.json (function calling)
                        └────► golden test suite (every example is executed in CI)
```

The binary is therefore **self-describing** - an agent can discover everything
at runtime without a bloated prompt:

```sh
ait meta catalog                    # full machine-readable catalog (JSON)
ait meta describe text case      # one tool: schema + verified examples
ait meta export --target all       # compile dist/ artifacts
ait meta parity                    # coverage vs upstream it-tools (456 tools)
```

Three invariants are enforced by `cargo test` (see `tests/spec_golden.rs`):

1. every CLI leaf command has a spec, and every spec has a CLI leaf
2. every spec example **runs** and produces the documented output
3. every upstream reference resolves in `specs/parity.toml`

So the skill file, the plugin, and the OpenAI tool definitions can never drift
from what the binary actually does.

### Adding a tool

1. Implement the subcommand in the right `src/<category>.rs` (mature crates
   only - no hand-rolled algorithms).
2. Write `specs/<category>/<tool>.toml` with at least one example.
3. Flip the upstream id in `specs/parity.toml` from `planned` to `implemented`.
4. `cargo test` - the drift/golden/parity suite tells you what's missing.
5. `ait meta export` to regenerate `dist/`.

## Distribution artifacts (`dist/`, generated)

- `skill/agent-it-tools/SKILL.md` - Claude skill (compact index + progressive
  disclosure via `meta describe`, tuned for small models)
- `claude-plugin/` - Claude Code plugin layout (`.claude-plugin/plugin.json` +
  skill), ready for a plugin marketplace
- `openai-tools.json` - OpenAI function-calling tool definitions (one function
  per tool, args mapped from the clap schema)
- `catalog.json` - the raw merged catalog for any other integration (MCP
  server mode can be generated from this later)

## Evals: does it help small models?

`evals/` contains an A/B harness that runs identical tasks through headless
`claude -p` in two arms - `bare` (no binary, no skill) vs `skill` (binary on
PATH + skill installed) - and grades deterministic expected answers. See
[`evals/README.md`](evals/README.md).

```sh
./evals/run.sh haiku both
```

## Command taxonomy

`ait <category> <tool> [args/flags]` - the category is the noun you operate on.
60 tools across 17 noun categories (`json`, `data`, `encode`, `decode`, `url`,
`jwt`, `crypto`, `generate`, `text`, `regex`, `time`, `http`, `color`,
`markdown`, `network`, `math`, `unix`; run `ait meta catalog` for the authoritative
list), covering 73 upstream it-tools; `specs/parity.toml` tracks the rest.

## Build

```sh
cargo build --release          # ~3 MB optimized binary (lto + strip)
cargo test                     # spec drift + golden examples + parity
# fully static Linux build:
cargo build --release --target x86_64-unknown-linux-musl
```

## Crate choices

Core algorithms are delegated to mature crates, never hand-rolled: `clap`
(derive), `anyhow`, `serde`/`serde_json`/`serde_yaml`/`toml`, `base64`,
`urlencoding`, `hex`, `sha1`/`sha2`/`md-5`/`hmac`/`bcrypt`, `slug`, `uuid`,
`rand`, `heck`, `cron`, `chrono`, `url`, `woothee`, `html-escape`, `regex`,
`similar`, `include_dir`.
