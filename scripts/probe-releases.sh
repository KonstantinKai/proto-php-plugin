#!/usr/bin/env bash
# Probe dl.static-php.dev for available PHP prebuilt binaries.
# Outputs a JSON file with verified version/platform availability.
set -euo pipefail

OUTPUT="${1:-releases.json}"
BASE_URL="https://dl.static-php.dev/static-php-cli/common"

platforms=("linux-x86_64" "linux-aarch64" "macos-x86_64" "macos-aarch64")

echo "Probing static-php-cli for available PHP versions..."
echo "{" > "$OUTPUT"
echo '  "generated": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",' >> "$OUTPUT"
echo '  "versions": {' >> "$OUTPUT"

first_version=true

# Probe 8.0.x through 8.5.x
for major_minor in 8.0 8.1 8.2 8.3 8.4 8.5; do
  for patch in $(seq 0 40); do
    ver="${major_minor}.${patch}"
    # Quick check with linux-x86_64
    code=$(curl -sI -o /dev/null -w "%{http_code}" "${BASE_URL}/php-${ver}-cli-linux-x86_64.tar.gz" 2>/dev/null || echo "000")

    if [ "$code" = "302" ] || [ "$code" = "200" ]; then
      # Found! Check all platforms
      if [ "$first_version" = true ]; then
        first_version=false
      else
        echo "," >> "$OUTPUT"
      fi

      printf '    "%s": [' "$ver" >> "$OUTPUT"
      first_platform=true

      for platform in "${platforms[@]}"; do
        pcode=$(curl -sI -o /dev/null -w "%{http_code}" "${BASE_URL}/php-${ver}-cli-${platform}.tar.gz" 2>/dev/null || echo "000")
        if [ "$pcode" = "302" ] || [ "$pcode" = "200" ]; then
          if [ "$first_platform" = true ]; then
            first_platform=false
          else
            printf ", " >> "$OUTPUT"
          fi
          printf '"%s"' "$platform" >> "$OUTPUT"
        fi
      done

      printf "]" >> "$OUTPUT"
      echo "  $ver: available" >&2
    fi
  done
done

echo "" >> "$OUTPUT"
echo "  }" >> "$OUTPUT"
echo "}" >> "$OUTPUT"

echo "Done! Written to $OUTPUT"
