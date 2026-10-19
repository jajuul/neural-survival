#!/usr/bin/env bash
# Check that every OpenSpec scenario is linked to a test that exists and runs.
#
# A scenario is linked by one or more lines inside its block:
#   <!-- verified-by: <path/to/file.rs>::<test_fn> -->
# Each link must name an existing file that defines `fn <test_fn>`, the
# function must be a test the workspace actually registers, and it must not be
# #[ignore]d. Whether the linked tests pass is the test suite's job.
#
# Usage: scripts/check-traceability.sh [<change-name>]
# Checks openspec/specs/ plus the delta specs of <change-name>, or of every
# active change when none is given. Scenarios under "## REMOVED Requirements"
# need no link. Exits non-zero when any scenario or link fails.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

if [[ $# -gt 0 ]]; then
    [[ -d "openspec/changes/$1" ]] || {
        echo "error: no active OpenSpec change '$1'" >&2
        exit 2
    }
    change_dirs=("openspec/changes/$1")
else
    change_dirs=()
    for dir in openspec/changes/*/; do
        [[ "$dir" == openspec/changes/archive/ ]] || change_dirs+=("${dir%/}")
    done
fi

spec_files=()
while IFS= read -r file; do
    spec_files+=("$file")
done < <(find openspec/specs "${change_dirs[@]/%//specs}" -name spec.md 2>/dev/null | sort)

if [[ ${#spec_files[@]} -eq 0 ]]; then
    echo "No specs to check."
    exit 0
fi

# One record per line: "SCENARIO <file>:<line>", "UNLINKED <file>:<line> <title>"
# or "LINK <file>:<line> <path>::<fn>".
records="$(awk '
    function close_scenario() {
        if (in_scenario && !linked) print "UNLINKED " FILENAME ":" start " " title
        in_scenario = 0
    }
    FNR == 1 { close_scenario(); removed = 0 }
    /^## / { close_scenario(); removed = ($0 ~ /^## REMOVED/) }
    /^### / { close_scenario() }
    /^#### Scenario:/ {
        close_scenario()
        if (!removed) {
            in_scenario = 1; linked = 0; start = FNR
            title = $0; sub(/^#### Scenario:[ \t]*/, "", title)
            print "SCENARIO " FILENAME ":" FNR
        }
        next
    }
    in_scenario && /<!-- *verified-by:/ {
        link = $0
        sub(/.*verified-by:[ \t]*/, "", link); sub(/[ \t]*-->.*/, "", link)
        print "LINK " FILENAME ":" FNR " " link
        linked = 1
    }
    END { close_scenario() }
' "${spec_files[@]}")"

list_tests() {
    cargo test --locked --workspace --quiet -- --list --format terse "$@" 2>/dev/null \
        | sed -n 's/: test$//p' | sed 's/.*:://' | sort -u
}
all_tests="$(list_tests)" || {
    echo "error: could not list the workspace's tests; does it compile?" >&2
    exit 2
}
ignored_tests="$(list_tests --ignored)"

problems=0
report() {
    echo "$1"
    problems=$((problems + 1))
}

while IFS= read -r record; do
    [[ -n "$record" ]] || continue
    read -r kind location rest <<< "$record"
    case "$kind" in
        UNLINKED)
            report "$location: scenario has no verified-by link: $rest"
            ;;
        LINK)
            path="${rest%%::*}"
            fn="${rest##*::}"
            if [[ "$rest" != *::* || -z "$path" || -z "$fn" ]]; then
                report "$location: malformed link '$rest' (expected <path>::<test_fn>)"
            elif [[ ! -f "$path" ]]; then
                report "$location: linked file does not exist: $path"
            elif ! grep -Eq "fn ${fn}[[:space:]]*\(" "$path"; then
                report "$location: $path does not define fn $fn"
            elif ! grep -qx "$fn" <<< "$all_tests"; then
                report "$location: $fn is not a registered test"
            elif grep -qx "$fn" <<< "$ignored_tests"; then
                report "$location: $fn is #[ignore]d"
            fi
            ;;
    esac
done <<< "$records"

scenarios="$(grep -c '^SCENARIO ' <<< "$records" || true)"
links="$(grep -c '^LINK ' <<< "$records" || true)"
echo "Traceability: $scenarios scenarios, $links links, $problems problems."
[[ $problems -eq 0 ]]
