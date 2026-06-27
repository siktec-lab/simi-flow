#!/usr/bin/env bash
# Version bumper for SIMI (cargo + pyproject + js + napi platform pins)
# Usage: bash scripts/bump-version.sh 0.2.0

set -euo pipefail

if [ -z "${1:-}" ]; then
    echo "Usage: $0 <new-version>"
    exit 1
fi

NEW_VERSION="$1"

# Validate semver-ish: X.Y.Z with optional -prerelease / +build
if ! printf '%s' "$NEW_VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)*$'; then
    echo "Error: '$NEW_VERSION' is not a valid version (expected X.Y.Z)" >&2
    exit 1
fi

# Resolve repo root so the script works from any cwd.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Portable in-place sed (GNU vs BSD/macOS).
sedi() {
    if sed --version >/dev/null 2>&1; then
        sed -i "$@"
    else
        sed -i '' "$@"
    fi
}

# Cargo.toml — package version (first ^version line under [package]).
sedi "0,/^version = \".*\"/s//version = \"$NEW_VERSION\"/" Cargo.toml

# pyproject.toml — project version.
sedi "0,/^version = \".*\"/s//version = \"$NEW_VERSION\"/" pyproject.toml

# js/package.json — top-level "version" key (first match only).
sedi "0,/\"version\": \".*\"/s//\"version\": \"$NEW_VERSION\"/" js/package.json

# js/package.json — keep the napi platform-package pins in optionalDependencies
# locked to the main version, or npm resolves no native binary at install time.
sedi -E "s|(\"@siktec-lab/simi-flow-[a-z0-9-]+\": )\"[^\"]*\"|\1\"$NEW_VERSION\"|g" js/package.json

# Generated platform packages (only exist in CI after `napi create-npm-dirs`).
for f in js/npm/*/package.json; do
    [ -f "$f" ] && sedi "0,/\"version\": \".*\"/s//\"version\": \"$NEW_VERSION\"/" "$f"
done

echo "Version bumped to $NEW_VERSION"
echo ""
echo "Updated:"
echo "  Cargo.toml, pyproject.toml, js/package.json (version + platform pins)"
echo ""
echo "Next steps:"
echo "  cargo build            # refresh Cargo.lock"
echo "  git add -A"
echo "  git commit -m \"Release v$NEW_VERSION\""
echo "  git tag v$NEW_VERSION"
echo "  git push --follow-tags"
