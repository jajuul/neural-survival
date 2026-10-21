#!/usr/bin/env bash
# Step 4 of the review workflow (docs/agents/review-workflow.md): run every
# automated check on the committed change and record what actually happened.
#
# Usage: scripts/review-gate.sh [--mutants] <base> [<change-name>]
#   --mutants      also run mutation testing, which is slow; without it the
#                  check is recorded SKIPPED
#   <base>         the commit the change started from; diffs use <base>...HEAD
#   <change-name>  the OpenSpec change under openspec/changes/; its proposal's
#                  "Tier:" line sets the tier. Without it the tier is light.
#
# Runs on committed work only: the working tree must be clean apart from
# review output. Every check runs even when an earlier one fails, except
# mutation testing, which needs passing tests. A check whose tool is missing
# is reported SKIPPED, never passed. Surviving mutants pass only when listed
# in openspec/changes/<change-name>/accepted-mutants, one per line as
# "<file>: <mutation>  # <ticket that will make it testable>", the format of
# mutants.out/missed.txt (a :line:col after the file is ignored).
# The report goes to stdout and, for a
# change, to openspec/changes/<change-name>/review/gate.md.
# Exits non-zero when any check fails.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"

usage() {
    echo "usage: scripts/review-gate.sh [--mutants] <base> [<change-name>]" >&2
    exit 2
}
mutants=0
args=()
for arg in "$@"; do
    case "$arg" in
        --mutants) mutants=1 ;;
        -*) usage ;;
        *) args+=("$arg") ;;
    esac
done
[[ ${#args[@]} -ge 1 && ${#args[@]} -le 2 ]] || usage
base="${args[0]}"
change="${args[1]:-}"

git rev-parse --verify --quiet "$base^{commit}" >/dev/null || {
    echo "error: unknown base '$base'" >&2
    exit 2
}
merge_base="$(git merge-base "$base" HEAD)"
head="$(git rev-parse --short HEAD)"

dirty="$(git status --porcelain | grep -Ev '^.. openspec/changes/[^/]+/review/' || true)"
if [[ -n "$dirty" ]]; then
    echo "error: commit or stash your changes first; the gate checks committed work only:" >&2
    echo "$dirty" >&2
    exit 2
fi

changed_files="$(git diff --name-only "$merge_base" HEAD)"
if [[ -z "$changed_files" ]]; then
    echo "error: no changes between $base and HEAD" >&2
    exit 2
fi

change_dir=""
declared_tier="light"
if [[ -n "$change" ]]; then
    change_dir="openspec/changes/$change"
    [[ -f "$change_dir/proposal.md" ]] || {
        echo "error: $change_dir/proposal.md not found" >&2
        exit 2
    }
    declared_tier="$(grep -ioE '^\**tier:?\**:?[[:space:]]*(light|standard|full)' "$change_dir/proposal.md" \
        | head -1 | grep -ioE '(light|standard|full)$' | tr '[:upper:]' '[:lower:]')"
    [[ -n "$declared_tier" ]] || {
        echo "error: $change_dir/proposal.md has no 'Tier: light|standard|full' line" >&2
        exit 2
    }
fi

tier_rank() {
    case "$1" in light) echo 0 ;; standard) echo 1 ;; full) echo 2 ;; esac
}

log_dir="$(mktemp -d)"
trap 'rm -rf "$log_dir"' EXIT
rows=()
details=()
failed=0

# record <check> <PASS|FAIL|SKIPPED> <exit code or -> <note> [<log file>]
record() {
    rows+=("| $1 | $2 | $3 | $4 |")
    if [[ "$2" == FAIL ]]; then
        failed=1
        if [[ -n "${5:-}" && -s "$5" ]]; then
            details+=("### $1" "" '```' "$(tail -40 "$5" | sed 's/\x1b\[[0-9;]*[A-Za-z]//g; s/\x1b(B//g')" '```' "")
        fi
    fi
}

# run <check> <command...>: run a command and record its outcome.
run() {
    local name="$1" log="$log_dir/$(tr -c 'a-z0-9\n' '-' <<< "$1").log" code
    shift
    "$@" > "$log" 2>&1
    code=$?
    if [[ $code -eq 0 ]]; then
        record "$name" PASS "$code" "\`$*\`"
    else
        record "$name" FAIL "$code" "\`$*\`" "$log"
    fi
    return "$code"
}

run "Formatting" cargo fmt --all --check
run "Clippy" cargo clippy --locked --workspace --all-targets -- -D warnings
run "Tests" cargo test --locked --workspace
tests_status=$?
run "Architecture" scripts/check-architecture.sh

if command -v openspec >/dev/null 2>&1; then
    run "OpenSpec validation" openspec validate --all --strict --no-interactive
else
    record "OpenSpec validation" SKIPPED - "openspec is not on PATH"
fi

if ! cargo deny --version >/dev/null 2>&1; then
    record "Dependency audit" SKIPPED - "cargo-deny is not installed"
elif [[ ! -f deny.toml ]]; then
    record "Dependency audit" SKIPPED - "no deny.toml"
else
    run "Dependency audit" cargo deny --locked check
fi

if [[ -n "$change" ]]; then
    run "Traceability" scripts/check-traceability.sh "$change"
else
    run "Traceability" scripts/check-traceability.sh
fi

# Approved acceptance tests must be exactly as they were when approved.
approval="$change_dir/acceptance-approved"
if [[ "$declared_tier" == light ]]; then
    record "Acceptance-test lock" SKIPPED - "light tier has no acceptance tests"
elif [[ ! -f "$approval" ]]; then
    record "Acceptance-test lock" FAIL - "$approval is missing: acceptance tests were never approved"
else
    lock_log="$log_dir/acceptance.log"
    approved_commit="$(sed -n 's/^commit[[:space:]]\+//p' "$approval" | head -1)"
    mapfile -t approved_files < <(grep -v '^commit[[:space:]]' "$approval" | grep -v '^[[:space:]]*\(#\|$\)')
    if [[ -z "$approved_commit" || ${#approved_files[@]} -eq 0 ]]; then
        echo "$approval needs a 'commit <sha>' line and at least one test file" > "$lock_log"
    elif ! git merge-base --is-ancestor "$approved_commit" HEAD 2>/dev/null; then
        echo "approved commit $approved_commit is not an ancestor of HEAD" > "$lock_log"
    else
        for file in "${approved_files[@]}"; do
            [[ -f "$file" ]] || echo "$file: deleted since approval" >> "$lock_log"
        done
        git diff --stat "$approved_commit" HEAD -- "${approved_files[@]}" >> "$lock_log"
    fi
    if [[ -s "$lock_log" ]]; then
        record "Acceptance-test lock" FAIL - "approved tests changed since approval" "$lock_log"
    else
        record "Acceptance-test lock" PASS - "${#approved_files[@]} file(s) unchanged since $approved_commit"
    fi
fi

# The diff may demand a higher tier than the proposal declared.
required_tier="$declared_tier"
tier_log="$log_dir/tier.log"
require() {
    echo "$2 -> $1" >> "$tier_log"
    if [[ $(tier_rank "$1") -gt $(tier_rank "$required_tier") ]]; then
        required_tier="$1"
    fi
}
always_read=()
if [[ -f docs/agents/always-read.txt ]]; then
    mapfile -t always_read < <(grep -v '^[[:space:]]*\(#\|$\)' docs/agents/always-read.txt)
fi
while IFS= read -r file; do
    for pattern in "${always_read[@]}"; do
        # shellcheck disable=SC2053 # the pattern is a glob on purpose
        [[ "$file" == $pattern ]] && require full "$file is an always-read path ($pattern)"
    done
    [[ "$file" == Cargo.toml || "$file" == */Cargo.toml || "$file" == Cargo.lock ]] \
        && require full "$file changes dependencies or crate setup"
done <<< "$changed_files"
if git diff "$merge_base" HEAD -- '*.rs' | grep -Eq '^-[^-].*assert'; then
    require standard "the diff removes or changes test assertions"
fi
if [[ $(tier_rank "$required_tier") -gt $(tier_rank "$declared_tier") ]]; then
    record "Tier" FAIL - "declared $declared_tier, the diff requires $required_tier" "$tier_log"
else
    record "Tier" PASS - "$declared_tier"
fi

# Mutation testing last: it is slow and meaningless while tests fail.
# A surviving (missed or timed-out) mutant fails the gate unless the human
# accepted it in <change>/accepted-mutants. Entries match on file and
# mutation, ignoring line and column, so unrelated edits don't break them.
normalize_mutants() {
    sed -E 's/[[:space:]]*#.*$//; s/^([^:]+):[0-9]+:[0-9]+: /\1: /; s/[[:space:]]+$//' \
        | grep -v '^$' | sort -u
}
accepted_survivors=""
if [[ $mutants -eq 0 ]]; then
    record "Mutation testing" SKIPPED - "not requested; rerun with --mutants"
elif [[ $tests_status -ne 0 ]]; then
    record "Mutation testing" SKIPPED - "tests failed"
else
    mutants_log="$log_dir/mutation-testing.log"
    scripts/mutants-ticket.sh "$merge_base" > "$mutants_log" 2>&1
    mutants_status=$?
    survivors=""
    if [[ $mutants_status -eq 2 || $mutants_status -eq 3 ]]; then
        survivors="$(cat mutants.out/missed.txt mutants.out/timeout.txt 2>/dev/null | normalize_mutants)"
    fi
    if [[ $mutants_status -eq 0 ]]; then
        record "Mutation testing" PASS 0 "no surviving mutants"
    elif [[ -z "$survivors" ]]; then
        record "Mutation testing" FAIL "$mutants_status" "cargo mutants did not complete" "$mutants_log"
    else
        accepted=""
        if [[ -n "$change_dir" && -f "$change_dir/accepted-mutants" ]]; then
            accepted="$(normalize_mutants < "$change_dir/accepted-mutants")"
        fi
        unaccepted="$(comm -23 <(echo "$survivors") <(echo "$accepted"))"
        accepted_survivors="$(comm -12 <(echo "$survivors") <(echo "$accepted"))"
        total=$(grep -c . <<< "$survivors")
        if [[ -z "$unaccepted" ]]; then
            record "Mutation testing" PASS "$mutants_status" "$total surviving mutant(s), all accepted"
        else
            echo "$unaccepted" > "$mutants_log"
            record "Mutation testing" FAIL "$mutants_status" \
                "$(grep -c . <<< "$unaccepted") of $total surviving mutant(s) not in accepted-mutants" \
                "$mutants_log"
        fi
    fi
fi

report="$(
    echo "# Review gate"
    echo
    echo "- Change: ${change:-none (light)}"
    echo "- Commits: $(git rev-parse --short "$merge_base")..$head"
    echo "- Tier: $declared_tier (required by the diff: $required_tier)"
    echo "- Run: $(date -Iseconds)"
    echo
    echo "| Check | Result | Exit code | Detail |"
    echo "|---|---|---|---|"
    printf '%s\n' "${rows[@]}"
    if [[ -n "$accepted_survivors" ]]; then
        echo
        echo "## Accepted surviving mutants"
        echo
        echo "Untested for now, accepted in accepted-mutants:"
        echo
        sed 's/^/- /' <<< "$accepted_survivors"
    fi
    if [[ ${#details[@]} -gt 0 ]]; then
        echo
        echo "## Failures"
        echo
        printf '%s\n' "${details[@]}"
    fi
)"

echo "$report"
if [[ -n "$change_dir" ]]; then
    mkdir -p "$change_dir/review"
    echo "$report" > "$change_dir/review/gate.md"
    echo
    echo "Written to $change_dir/review/gate.md"
fi
exit "$failed"
