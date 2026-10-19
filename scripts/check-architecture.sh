#!/usr/bin/env bash
# Check the architecture constraints the compiler and tests don't cover:
#   - the core crate has no GUI dependency (ADR-0003)
#   - every crate root forbids unsafe code
# Prints one line per violation; exits non-zero when any constraint fails.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

status=0

gui_crates='^(eframe|egui|egui_plot|egui_extras|winit|wgpu|glow|glutin|bevy|macroquad) '
gui_deps="$(cargo tree --locked -p neural-survival-core -e normal,build --prefix none \
    | grep -E "$gui_crates" | sort -u || true)"
if [[ -n "$gui_deps" ]]; then
    echo "ADR-0003: neural-survival-core depends on GUI crates:"
    sed 's/^/  /' <<< "$gui_deps"
    status=1
fi

for root in src/main.rs crates/*/src/lib.rs crates/*/src/main.rs; do
    [[ -f "$root" ]] || continue
    if ! grep -q '^#!\[forbid(unsafe_code)\]' "$root"; then
        echo "$root: missing #![forbid(unsafe_code)]"
        status=1
    fi
done

exit "$status"
