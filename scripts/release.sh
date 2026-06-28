#!/usr/bin/env bash
#
# Release helper for SIMI.
#
# `main` is protected: no direct pushes, PRs need CI + review, and only the
# owner may create `v*` tags. A release therefore happens in two phases:
#
#   1. `pr`  — bump the version on a branch and open a pull request.
#   2. `tag` — after that PR is merged, tag main so the Release workflow runs.
#
# Usage:
#   bash scripts/release.sh pr  <version>   # phase 1: branch + bump + PR
#   bash scripts/release.sh tag <version>   # phase 2: tag merged main -> publish
#   bash scripts/release.sh      <version>  # convenience: runs `pr`
#
# Requires: git, gh (authenticated), cargo. Run from anywhere in the repo.

set -euo pipefail

# ── helpers ─────────────────────────────────────────────────────────────
die()  { echo "Error: $*" >&2; exit 1; }
info() { echo ">> $*"; }

command -v git >/dev/null || die "git not found on PATH"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DEFAULT_BRANCH="main"

validate_version() {
    printf '%s' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)*$' \
        || die "'$1' is not a valid version (expected X.Y.Z)"
}

# ── phase 1: open the release PR ────────────────────────────────────────
do_pr() {
    local version="$1"
    validate_version "$version"
    command -v gh >/dev/null || die "gh (GitHub CLI) not found on PATH — needed to open the PR"
    local branch="release/v$version"

    [ -z "$(git status --porcelain)" ] \
        || die "working tree is dirty — commit or stash first"

    info "Syncing $DEFAULT_BRANCH"
    git checkout "$DEFAULT_BRANCH" >/dev/null 2>&1 \
        || die "could not checkout $DEFAULT_BRANCH"
    git pull --ff-only origin "$DEFAULT_BRANCH"

    if git rev-parse --verify "$branch" >/dev/null 2>&1; then
        die "branch '$branch' already exists — delete it or pick another version"
    fi

    info "Creating branch $branch"
    git checkout -b "$branch"

    info "Bumping version to $version"
    bash scripts/bump-version.sh "$version" >/dev/null

    info "Refreshing Cargo.lock"
    cargo build --quiet 2>/dev/null || cargo build

    if [ -z "$(git status --porcelain)" ]; then
        git checkout "$DEFAULT_BRANCH" >/dev/null 2>&1
        git branch -D "$branch" >/dev/null 2>&1
        die "no changes produced — is the repo already at $version?"
    fi

    git add -A
    git commit -q -m "release: v$version"

    info "Pushing $branch and opening a PR"
    git push -u origin "$branch"
    gh pr create \
        --base "$DEFAULT_BRANCH" \
        --head "$branch" \
        --title "release: v$version" \
        --body "Bump version to \`$version\` (Cargo, pyproject, npm + platform pins).

After this PR is merged, run:

    bash scripts/release.sh tag $version

to tag \`$DEFAULT_BRANCH\` and trigger the Release workflow."

    echo
    info "PR opened. Once CI passes and it is merged, run:"
    echo "    bash scripts/release.sh tag $version"
}

# ── phase 2: tag merged main and trigger the release ────────────────────
do_tag() {
    local version="$1"
    validate_version "$version"
    local tag="v$version"

    info "Syncing $DEFAULT_BRANCH"
    git checkout "$DEFAULT_BRANCH" >/dev/null 2>&1 \
        || die "could not checkout $DEFAULT_BRANCH"
    git pull --ff-only origin "$DEFAULT_BRANCH"

    # Confirm the bump actually landed on main before tagging.
    local main_version
    main_version="$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')"
    [ "$main_version" = "$version" ] \
        || die "$DEFAULT_BRANCH is at version '$main_version', not '$version' — has the release PR been merged?"

    if git ls-remote --tags origin | grep -q "refs/tags/$tag$"; then
        die "tag $tag already exists on origin — pick a new version"
    fi

    info "Tagging $tag on $(git rev-parse --short HEAD)"
    git tag -a "$tag" -m "Release $tag"
    git push origin "$tag"

    echo
    info "Tag pushed. Watch the release run with:"
    echo "    gh run watch \$(gh run list --workflow=release.yml --limit 1 --json databaseId --jq '.[0].databaseId') --exit-status"
}

# ── dispatch ────────────────────────────────────────────────────────────
case "${1:-}" in
    pr)   [ $# -eq 2 ] || die "usage: $0 pr <version>";  do_pr  "$2" ;;
    tag)  [ $# -eq 2 ] || die "usage: $0 tag <version>"; do_tag "$2" ;;
    "")   die "usage: $0 {pr|tag} <version>" ;;
    *)
        # Bare version (no subcommand) -> default to phase 1.
        validate_version "$1"
        do_pr "$1"
        ;;
esac
