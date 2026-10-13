#!/usr/bin/env bash
# Mutation-test a ticket's changes: every crate's src/ plus the root binary's
# src/, diffed against <base> (default HEAD, which covers uncommitted work).
# Untracked source files count as changes. Prints only missed or timed-out
# mutants and the summary; exits non-zero when any mutant survives.
set -euo pipefail

base="${1:-HEAD}"
cd "$(git rev-parse --show-toplevel)"

git rev-parse --verify --quiet "$base^{commit}" >/dev/null || {
    echo "error: unknown base '$base'" >&2
    exit 2
}

cargo mutants --version >/dev/null 2>&1 || {
    echo "error: cargo-mutants is missing; install it with: cargo install --locked cargo-mutants" >&2
    exit 2
}

diff_file="$(mktemp)"
log_file="$(mktemp)"
trap 'rm -f "$diff_file" "$log_file"' EXIT

paths=(crates/*/src src)
git diff "$base" -- "${paths[@]}" > "$diff_file"
while IFS= read -r file; do
    # --no-index exits 1 when the files differ, which they always do here.
    git diff --no-index -- /dev/null "$file" >> "$diff_file" || true
done < <(git ls-files --others --exclude-standard -- "${paths[@]}")

if [[ ! -s "$diff_file" ]]; then
    echo "No source changes since $base."
    exit 0
fi
echo "Files under test:"
grep '^+++ b/' "$diff_file" | sed 's|^+++ b/|  |'

status=0
cargo mutants --workspace --in-diff "$diff_file" > "$log_file" 2>&1 || status=$?
grep -E '^(MISSED|TIMEOUT)|mutants tested|^error' "$log_file" || tail -20 "$log_file"
exit "$status"
