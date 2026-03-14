#!/bin/bash

set -euo pipefail

repo="${1:-stakeswky/golutra-cli}"
commit_message="${2:-Extract standalone golutra-cli repo}"

gh_api_retry() {
  local attempt=1
  local max_attempts=5
  local output
  local status

  while true; do
    if output="$(gh api "$@" 2>&1)"; then
      printf '%s' "$output"
      return 0
    fi

    status=$?
    if [ "$attempt" -ge "$max_attempts" ]; then
      printf '%s\n' "$output" >&2
      return "$status"
    fi

    sleep "$attempt"
    attempt=$((attempt + 1))
  done
}

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

entries_jsonl="$tmp_dir/entries.jsonl"
tree_payload="$tmp_dir/tree.json"
commit_payload="$tmp_dir/commit.json"

if base_commit_sha="$(gh_api_retry "repos/$repo/git/ref/heads/main" --jq .object.sha 2>/dev/null)"; then
  :
else
  seed_content_base64="$(base64 < README.md | tr -d '\n')"
  gh_api_retry \
    -X PUT \
    "repos/$repo/contents/README.md" \
    -f message="Seed repository" \
    -f content="$seed_content_base64" \
    -f branch="main" >/dev/null
  base_commit_sha="$(gh_api_retry "repos/$repo/git/ref/heads/main" --jq .object.sha)"
fi

git ls-files --stage | while read -r mode _sha _stage path; do
  content_base64="$(base64 < "$path" | tr -d '\n')"
  blob_sha="$(
    gh_api_retry \
      -X POST \
      "repos/$repo/git/blobs" \
      -f encoding=base64 \
      -f content="$content_base64" \
      --jq .sha
  )"

  jq -cn \
    --arg path "$path" \
    --arg mode "$mode" \
    --arg sha "$blob_sha" \
    '{path: $path, mode: $mode, type: "blob", sha: $sha}' >> "$entries_jsonl"
done

jq -s '{tree: .}' "$entries_jsonl" > "$tree_payload"
tree_sha="$(gh_api_retry -X POST "repos/$repo/git/trees" --input "$tree_payload" --jq .sha)"

jq -cn \
  --arg message "$commit_message" \
  --arg tree "$tree_sha" \
  --arg parent "$base_commit_sha" \
  '{message: $message, tree: $tree, parents: [$parent]}' > "$commit_payload"

commit_sha="$(gh_api_retry -X POST "repos/$repo/git/commits" --input "$commit_payload" --jq .sha)"

if gh_api_retry "repos/$repo/git/ref/heads/main" >/dev/null 2>&1; then
  gh_api_retry -X PATCH "repos/$repo/git/refs/heads/main" -f sha="$commit_sha" -f force=true >/dev/null
else
  gh_api_retry -X POST "repos/$repo/git/refs" -f ref="refs/heads/main" -f sha="$commit_sha" >/dev/null
fi

printf 'Published commit %s to https://github.com/%s\n' "$commit_sha" "$repo"
