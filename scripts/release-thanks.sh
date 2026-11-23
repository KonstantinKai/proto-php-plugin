#!/usr/bin/env bash
# Print a "Thanks" section crediting outside contributors to a release, or
# nothing when every commit in the release came from the repository owner or bots.
#
# Usage: scripts/release-thanks.sh <ref>
#   ref  tag or commit that ends the release, e.g. v0.6.0
#
# The release spans from the previous tag reachable from <ref> to <ref>. Needs
# full git history (actions/checkout with fetch-depth: 0) and an authenticated
# gh CLI (GH_TOKEN in Actions). Commits without a linked GitHub account are skipped.
set -euo pipefail

ref="${1:?usage: $0 <ref>}"
repo="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
owner="${repo%%/*}"

if prev="$(git describe --tags --abbrev=0 "${ref}^" 2>/dev/null)"; then
  range="${prev}..${ref}"
else
  range="$ref"
fi

# One "login pr" line per contributor commit and linked PR ("login -" when the
# commit has no PR), grouped below. Plain lines keep this working on bash 3.2.
pairs=""

for sha in $(git rev-list --no-merges --reverse "$range"); do
  author="$(gh api "repos/$repo/commits/$sha" \
    --jq '"\(.author.login // "-") \(.author.type // "-")"')"
  login="${author% *}"
  type="${author#* }"

  if [ "$login" = "-" ] || [ "$login" = "$owner" ] || [ "$type" = "Bot" ]; then
    continue
  fi

  pairs="$pairs$login -"$'\n'
  numbers="$(gh api "repos/$repo/commits/$sha/pulls" --jq '.[].number')"
  # The endpoint above misses some PRs opened from forks; commit search finds them.
  if [ -z "$numbers" ]; then
    numbers="$(gh api "search/issues?q=repo:$repo+is:pr+$sha" --jq '.items[].number')"
  fi
  for pr in $numbers; do
    pairs="$pairs$login $pr"$'\n'
  done
done

[ -z "$pairs" ] && exit 0

printf '### Thanks\n\n'
printf '%s' "$pairs" | awk -v repo="$repo" '
  !($1 in links) { order[++n] = $1; links[$1] = "" }
  $2 != "-" && !(($1, $2) in seen) {
    seen[$1, $2] = 1
    link = "[#" $2 "](https://github.com/" repo "/pull/" $2 ")"
    links[$1] = links[$1] == "" ? link : links[$1] ", " link
  }
  END {
    for (i = 1; i <= n; i++) {
      login = order[i]
      printf "- @%s%s\n", login, (links[login] == "" ? "" : " (" links[login] ")")
    }
  }
'
