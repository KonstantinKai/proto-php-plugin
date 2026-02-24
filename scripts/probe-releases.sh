#!/usr/bin/env bash
# Probe dl.static-php.dev for available PHP prebuilt binaries.
# Outputs a JSON file with verified version/platform availability and SHA-256 checksums.
# Reuses existing checksums from previous runs to avoid re-downloading.
set -euo pipefail

OUTPUT="${1:-releases.json}"
BASE_URL="https://dl.static-php.dev/static-php-cli/common"

platforms=("linux-x86_64" "linux-aarch64" "macos-x86_64" "macos-aarch64")

# Read existing checksum from previous releases.json
get_existing_checksum() {
  local ver="$1" plat="$2"
  if [ -f "$OUTPUT" ] && command -v jq &>/dev/null; then
    jq -r --arg v "$ver" --arg p "$plat" '.versions[$v][$p] // empty' "$OUTPUT" 2>/dev/null || true
  fi
}

echo "Probing static-php-cli for available PHP versions..."

# Build versions JSON object incrementally
versions_json="{}"

for major_minor in 8.0 8.1 8.2 8.3 8.4 8.5; do
  for patch in $(seq 0 40); do
    ver="${major_minor}.${patch}"
    # Quick check with linux-x86_64
    code=$(curl -sI -o /dev/null -w "%{http_code}" "${BASE_URL}/php-${ver}-cli-linux-x86_64.tar.gz" 2>/dev/null || echo "000")

    if [ "$code" = "302" ] || [ "$code" = "200" ]; then
      echo "  $ver: available" >&2
      version_obj="{}"

      for platform in "${platforms[@]}"; do
        pcode=$(curl -sI -o /dev/null -w "%{http_code}" "${BASE_URL}/php-${ver}-cli-${platform}.tar.gz" 2>/dev/null || echo "000")
        if [ "$pcode" = "302" ] || [ "$pcode" = "200" ]; then
          existing=$(get_existing_checksum "$ver" "$platform")
          if [ -n "$existing" ]; then
            sha="$existing"
            echo "    $platform: reused" >&2
          else
            url="${BASE_URL}/php-${ver}-cli-${platform}.tar.gz"
            sha=$(curl -sSL "$url" | sha256sum | awk '{print $1}')
            echo "    $platform: computed $sha" >&2
          fi
          version_obj=$(echo "$version_obj" | jq --arg p "$platform" --arg s "$sha" '. + {($p): $s}')
        fi
      done

      versions_json=$(echo "$versions_json" | jq --arg v "$ver" --argjson p "$version_obj" '. + {($v): $p}')
    fi
  done
done

jq -n --arg gen "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --argjson v "$versions_json" \
  '{generated: $gen, versions: $v}' > "${OUTPUT}.tmp"
mv "${OUTPUT}.tmp" "$OUTPUT"

echo "Done! Written to $OUTPUT"
