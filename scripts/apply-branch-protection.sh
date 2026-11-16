#!/bin/sh
# Applies the branch protection declared in .github/rulesets/main.json to the
# GitHub repository: pull requests only, one approving review, review by the
# code owners, resolved conversations, every CI check green, no force-push, no
# deletion, and no bypass for administrators.
#
# This has to be run by a person with admin rights on the repository; nothing
# in the repository can apply it to itself. It needs the GitHub CLI, signed in.
#
#   sh scripts/apply-branch-protection.sh OWNER/REPO
#
# Run it again after editing the ruleset: it updates the existing one.
# Before the first run, replace the placeholder team handles in
# .github/CODEOWNERS with real teams, or "require review from code owners"
# has nobody to ask.
set -eu
repo="${1:?usage: apply-branch-protection.sh OWNER/REPO}"
file="$(dirname "$0")/../.github/rulesets/main.json"
if grep -q '@loandesk/' "$(dirname "$0")/../.github/CODEOWNERS"; then
  echo "warning: .github/CODEOWNERS still has the placeholder @loandesk/* teams" >&2
fi
id="$(gh api "repos/$repo/rulesets" --jq '.[] | select(.name == "main") | .id' | head -n 1)"
if [ -n "$id" ]; then
  gh api --method PUT "repos/$repo/rulesets/$id" --input "$file" --jq '"updated ruleset \(.id): \(.name)"'
else
  gh api --method POST "repos/$repo/rulesets" --input "$file" --jq '"created ruleset \(.id): \(.name)"'
fi
