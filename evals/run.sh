#!/usr/bin/env bash
# Agent-usability eval: does the CLI + skill measurably help a (small) model?
#
# Usage:  ./evals/run.sh [model] [arm] [runs]
#   model  claude model alias or id (default: haiku)
#   arm    skill | bare | both (default: both)
#   runs   repetitions per task (default: 1). A task only counts as PASS
#          when ALL runs pass: reliability, not one-shot luck, is the metric.
#
# Arms:
#   bare   headless claude with no tools at all: the model's own knowledge.
#   skill  headless claude in a workspace with .claude/skills/agent-it-tools
#          installed and the release binary on PATH.
#
# Both arms run with --setting-sources project so the user's global ~/.claude
# config (permissions, skills, CLAUDE.md) cannot leak into the measurement.
#
# Requires: claude CLI, jq, a release build, and exported artifacts:
#   cargo build --release && ./target/release/agent-it-tools meta export --target skill

set -euo pipefail
cd "$(dirname "$0")"
ROOT="$(cd .. && pwd)"

MODEL="${1:-haiku}"
ARM="${2:-both}"
RUNS="${3:-1}"
BIN_DIR="$ROOT/target/release"
SKILL_SRC="$ROOT/dist/skill/agent-it-tools"

[ -x "$BIN_DIR/agent-it-tools" ] || { echo "missing release binary - run: cargo build --release" >&2; exit 1; }
[ -f "$SKILL_SRC/SKILL.md" ] || { echo "missing skill - run: ./target/release/agent-it-tools meta export --target skill" >&2; exit 1; }

WORK="$(mktemp -d /tmp/agent-it-tools-eval.XXXXXX)"
RESULTS="$WORK/results"
mkdir -p "$WORK/bare" "$WORK/skill/.claude/skills" "$RESULTS"
cp -R "$SKILL_SRC" "$WORK/skill/.claude/skills/"

run_arm() {
    local arm="$1"
    local pass=0 total=0
    echo ""
    echo "## arm: $arm (model: $MODEL, runs per task: $RUNS)"
    printf "%-15s %-6s %6s %8s %6s %10s\n" task result runs turns ms cost_usd
    local n i
    n=$(jq length tasks.json)
    for ((i = 0; i < n; i++)); do
        # Fields are read individually (not @tsv) so regex backslashes survive.
        local id prompt expect
        id=$(jq -r ".[$i].id" tasks.json)
        prompt=$(jq -r ".[$i].prompt" tasks.json)
        expect=$(jq -r ".[$i].expect_regex" tasks.json)
        total=$((total + 1))
        local dir="$WORK/$arm" path_prefix=""
        [ "$arm" = "skill" ] && path_prefix="$BIN_DIR:"
        local tool_flags=(--allowedTools "Bash(agent-it-tools:*),Skill")
        [ "$arm" = "bare" ] && tool_flags=(--disallowedTools "Bash,Skill")
        local ok=0 r out_file
        for ((r = 1; r <= RUNS; r++)); do
            out_file="$RESULTS/$arm-$id-r$r.json"
            (
                cd "$dir"
                PATH="${path_prefix}${PATH}" claude -p "$prompt" \
                    --model "$MODEL" \
                    --output-format json \
                    --max-turns 8 \
                    --setting-sources project \
                    "${tool_flags[@]}" \
                    </dev/null >"$out_file" 2>"${out_file%.json}.err" || true
            )
            if jq -e --arg re "$expect" '(.result // "") | test($re)' "$out_file" >/dev/null 2>&1; then
                ok=$((ok + 1))
            fi
        done
        local turns ms cost verdict=FAIL
        turns="$(jq -r '.num_turns // "?"' "$out_file" 2>/dev/null || echo "?")"
        ms="$(jq -r '.duration_ms // "?"' "$out_file" 2>/dev/null || echo "?")"
        cost="$(jq -r '.total_cost_usd // "?"' "$out_file" 2>/dev/null || echo "?")"
        [ "$ok" -eq "$RUNS" ] && { verdict=PASS; pass=$((pass + 1)); }
        printf "%-15s %-6s %6s %8s %6s %10s\n" "$id" "$verdict" "$ok/$RUNS" "$turns" "$ms" "$cost"
    done
    echo "score: $pass/$total (a task passes only if all $RUNS runs pass)"
}

case "$ARM" in
    bare) run_arm bare ;;
    skill) run_arm skill ;;
    both) run_arm bare; run_arm skill ;;
    *) echo "unknown arm '$ARM' (expected skill, bare or both)" >&2; exit 1 ;;
esac

echo ""
echo "raw transcripts: $RESULTS"
