// Version bumper for SIMI (cargo + pyproject + js)
// Usage: bash scripts/bump-version.sh 0.2.0

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <new-version>"
    exit 1
fi

NEW_VERSION="$1"

# Update Cargo.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update pyproject.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" pyproject.toml

# Update js/package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" js/package.json

# Update js/npm/*/package.json if they exist
for f in js/npm/*/package.json; do
    [ -f "$f" ] && sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" "$f"
done

echo "Version bumped to $NEW_VERSION"
echo ""
echo "Updated files:"
echo "  Cargo.toml"
echo "  pyproject.toml"
echo "  js/package.json"
echo ""
echo "Next steps:"
echo "  git add -A"
echo '  git commit -m "Release v'$NEW_VERSION'"'
echo "  git tag v$NEW_VERSION"
echo "  git push --follow-tags"
