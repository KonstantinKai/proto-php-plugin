#!/usr/bin/env bash
# Probe dl.static-php.dev for available PHP prebuilt binaries.
# Outputs a JSON file with verified version/platform availability and SHA-256 checksums.
# NOTE: Checksums are always recomputed because static-php-cli may rebuild binaries in-place.
set -euo pipefail

OUTPUT="${1:-releases.json}"
BASE_URL="https://dl.static-php.dev/static-php-cli/common"

platforms=("linux-x86_64" "linux-aarch64" "macos-x86_64" "macos-aarch64")

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
          url="${BASE_URL}/php-${ver}-cli-${platform}.tar.gz"
          sha=$(curl -sSL "$url" | sha256sum | awk '{print $1}')
          echo "    $platform: $sha" >&2
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
