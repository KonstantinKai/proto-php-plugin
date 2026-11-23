#!/usr/bin/env bash
# Print the CHANGELOG.md section for one version, for use as a GitHub release body.
#
# Usage: scripts/release-notes.sh [--thanks] <version> [changelog]
#   --thanks   append a "Thanks" section for outside contributors (see release-thanks.sh)
#   version    0.5.0 or v0.5.0
#   changelog  defaults to CHANGELOG.md
#
# Exits non-zero when the version has no section or the section is empty, so a
# release is never published with blank notes.
set -euo pipefail

thanks=false
if [ "${1:-}" = "--thanks" ]; then
  thanks=true
  shift
fi

version="${1:?usage: $0 [--thanks] <version> [changelog]}"
version="${version#v}"
changelog="${2:-CHANGELOG.md}"

notes="$(awk -v ver="$version" '
  /^## \[/ {
    if (found) exit
    if (index($0, "## [" ver "]") == 1) { found = 1; next }
  }
  # Reference-style link definitions at the bottom belong to no section.
  found && /^\[[^]]+\]: / { exit }
  found { print }
' "$changelog")"

# Trim leading and trailing blank lines.
notes="$(printf '%s\n' "$notes" | sed -e '/./,$!d' | sed -e ':a' -e '/^\n*$/{$d;N;ba' -e '}')"

if [ -z "$notes" ]; then
  echo "error: no CHANGELOG entry for version $version in $changelog" >&2
  exit 1
fi

link="$(grep -E "^\[$version\]: " "$changelog" | sed -E 's/^\[[^]]+\]: //' || true)"

printf '%s\n' "$notes"
if [ "$thanks" = true ]; then
  credits="$("$(dirname "$0")/release-thanks.sh" "v$version")"
  if [ -n "$credits" ]; then
    printf '\n%s\n' "$credits"
  fi
fi
if [ -n "$link" ]; then
  printf '\n**Full Changelog**: %s\n' "$link"
fi
