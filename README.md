# agent-it-tools

**Language models predict. This binary computes.**

Ask a model for a sha256 digest, an HMAC signature, or a base64 decode and it
will answer fluently, whether or not the answer is real. Sometimes it is. And
for an agent pipeline, *sometimes* is just wrong: a fabricated digest is
byte-for-byte indistinguishable from a correct one to every downstream step.
Inconsistent capability is not a weaker form of capability. It is noise
wearing the costume of signal.

`agent-it-tools` is a single static Rust binary that gives agents the
deterministic version of the [it-tools](https://github.com/sharevb/it-tools)
developer utility suite: hashing, HMAC, encodings, data-format conversion,
JWT/URL/user-agent parsing, cron, regex, diffs. Every answer is computed, the
same way, every time.

Measured on the smallest current Claude model (Haiku), same ten tasks,
identical prompts:

| | bare model | model + agent-it-tools |
|---|---|---|
| score | 8/10, unstable across runs | **10/10, stable** |
| sha256 / HMAC | confidently fabricated hex | computed |
| cost per task | comparable | ~$0.02 |

The bare 8/10 flatters the model: outside a few memorized facts those passes
are coin-flips that change from run to run (see [`evals/`](evals/README.md)).
The tool arm's only failure mode is not invoking the tool, and the shipped
skill file exists to drive exactly that to zero.

## Agent-first I/O contract

- **Success** → result written directly to `stdout` (plain text, or pretty
  JSON for structured tools). No spinners, colors, or ASCII art.
- **Failure** → `{"error": "reason"}` as valid JSON on `stderr`, non-zero exit
  code (`1` for errors; `crypto bcrypt-verify` exits `2` on a mismatch).
- Every tool takes its main body input as an **optional positional argument**;
  when omitted, it reads the full **stdin** pipe instead:

```sh
agent-it-tools crypto hash --algo sha256 "hello"
cat payload.json | agent-it-tools converter data --from json --to yaml
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
agent-it-tools meta catalog                    # full machine-readable catalog (JSON)
agent-it-tools meta describe converter case    # one tool: schema + verified examples
agent-it-tools meta export --target all       # compile dist/ artifacts
agent-it-tools meta parity                    # coverage vs upstream it-tools (456 tools)
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
5. `agent-it-tools meta export` to regenerate `dist/`.

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

`agent-it-tools <category> <tool> [args/flags]` - mirrors the it-tools layout.
27 tools across `crypto`, `converter`, `web`, `development` (run
`agent-it-tools meta catalog` for the authoritative list), covering 29 upstream
it-tools so far; `specs/parity.toml` tracks the remaining ~420.

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
