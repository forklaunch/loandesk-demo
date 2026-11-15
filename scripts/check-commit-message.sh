#!/bin/sh
# Commit message convention, enforced by .husky/commit-msg and in CI:
#   - subject line: starts with a capital letter, at most 72 characters, no trailing period
#   - a blank line between subject and body
# Usage: check-commit-message.sh <file containing the message>
set -eu
subject="$(sed -n 1p "$1")"
second="$(sed -n 2p "$1")"
fail() { echo "commit message: $1" >&2; echo "  subject: $subject" >&2; exit 1; }
case "$subject" in
  Merge\ *|Revert\ *|fixup!\ *|squash!\ *) exit 0 ;;
esac
[ -n "$subject" ] || fail "the subject line is empty"
[ "${#subject}" -le 72 ] || fail "the subject line is longer than 72 characters"
case "$subject" in *.) fail "the subject line ends with a period" ;; esac
case "$subject" in [A-Z]*) ;; *) fail "the subject line must start with a capital letter" ;; esac
[ -z "$second" ] || fail "the second line must be blank"
