#!/usr/bin/env bash

set -euo pipefail

COMMIT_MSG_FILE="${1:-}"

if [[ -z "$COMMIT_MSG_FILE" || ! -f "$COMMIT_MSG_FILE" ]]; then
  echo "Error: commit message file was not provided."
  exit 1
fi

# 先頭行だけを検査する
COMMIT_MSG="$(head -n 1 "$COMMIT_MSG_FILE")"

# merge / revert / fixup / squash は許可
if [[ "$COMMIT_MSG" =~ ^Merge\  ]] ||
   [[ "$COMMIT_MSG" =~ ^Revert\  ]] ||
   [[ "$COMMIT_MSG" =~ ^fixup!\  ]] ||
   [[ "$COMMIT_MSG" =~ ^squash!\  ]]; then
  exit 0
fi

# Conventional Commits
# 例:
# feat: add login page
# fix(api): handle timeout
# feat!: remove legacy API
# feat(api)!: remove legacy endpoint
PATTERN='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-zA-Z0-9._/-]+\))?(!)?: .+'

if [[ ! "$COMMIT_MSG" =~ $PATTERN ]]; then
  cat <<EOF
Invalid commit message:

  $COMMIT_MSG

Expected format:

  <type>(<scope>): <description>

Examples:

  feat: add user authentication
  fix(api): handle request timeout
  docs: update installation guide
  refactor(core): simplify validation
  feat(api)!: remove legacy endpoint

Allowed types:

  feat, fix, docs, style, refactor, perf,
  test, build, ci, chore, revert
EOF

  exit 1
fi
