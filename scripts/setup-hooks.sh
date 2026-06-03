#!/bin/sh
# Install the project's git hooks.
#
# Hooks live in the tracked .githooks/ directory; this points git at them via
# core.hooksPath (a local, per-clone setting that isn't committed) and makes
# them executable. Run once after cloning:
#
#   ./scripts/setup-hooks.sh
#
set -e

# Always operate from the repository root, regardless of where this is invoked.
cd "$(git rev-parse --show-toplevel)"

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit .githooks/pre-push

echo "✓ Git hooks installed (core.hooksPath -> .githooks)"
echo "  pre-commit: cargo fmt --check"
echo "  pre-push:   cargo clippy"
