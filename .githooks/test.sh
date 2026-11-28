#!/bin/sh
# Tests of the git hooks (ADR-0041, decision 7): a throwaway repository with the hooks
# installed, a planted private term and a planted fake secret. Needs what the hooks need.
# The fake secret is assembled at run time, so it never appears in this file.

set -u
SOURCE=$(cd "$(dirname "$0")" && pwd)
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
TERM_TEXT=Quokkafield
PASSED=0
FAILED=0

ok() { PASSED=$((PASSED + 1)); printf 'ok      %s\n' "$1"; }
not_ok() { FAILED=$((FAILED + 1)); printf 'NOT OK  %s\n' "$1"; }

# A fresh repository with the hooks installed and a bare remote.
setup() {
    rm -rf "$WORK/repo" "$WORK/remote.git"
    git init -q --bare "$WORK/remote.git"
    git init -q "$WORK/repo"
    cd "$WORK/repo" || exit 1
    git config user.email hooks@example.com
    git config user.name "hook test"
    git config commit.gpgsign false
    cp -R "$SOURCE" .githooks
    git config core.hooksPath .githooks
    printf '# test terms\n\n%s\n' "$TERM_TEXT" >.private-terms
    printf '.private-terms\n' >.gitignore
    git add .gitignore .githooks
    git commit -q --no-verify -m "initial commit"
    git remote add origin "$WORK/remote.git"
    git push -q --no-verify origin HEAD:main
}

fake_secret() {
    printf 'ghp_%s' "$(LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 36)"
}

# expect <refuse|accept> <name> <command...>: runs the command, checks the outcome, and
# checks that the output never contains the private term.
expect() {
    want=$1 name=$2
    shift 2
    if "$@" >"$WORK/out" 2>&1; then got=accept; else got=refuse; fi
    if grep -q -i -F "$TERM_TEXT" "$WORK/out"; then
        not_ok "$name: the output shows the private term"
    elif [ "$got" = "$want" ]; then
        ok "$name"
    else
        not_ok "$name: expected $want, got $got"
        sed 's/^/        /' "$WORK/out"
    fi
}

setup
printf 'hello\n' >clean.txt && git add clean.txt
expect accept "a clean commit passes" git commit -q -m "add a clean file"
expect accept "a clean push passes" git push -q origin HEAD:main

setup
printf 'line one\nmet %s yesterday\n' "$(printf '%s' "$TERM_TEXT" | tr '[:lower:]' '[:upper:]')" \
    >notes.txt && git add notes.txt
expect refuse "a private term in a staged line is refused, ignoring case" \
    git commit -q -m "add notes"

setup
printf 'x\n' >"$TERM_TEXT.txt" && git add "$TERM_TEXT.txt"
expect refuse "a private term in a staged path is refused" git commit -q -m "add a file"

setup
printf 'x\n' >file.txt && git add file.txt
expect refuse "a private term in the commit message is refused" \
    git commit -q -m "lunch with $TERM_TEXT"

setup
printf 'token = "%s"\n' "$(fake_secret)" >config.txt && git add config.txt
expect refuse "a secret in a staged line is refused" git commit -q -m "add config"

setup
printf 'x\n' >file.txt && git add file.txt
expect refuse "a secret in the commit message is refused" \
    git commit -q -m "token $(fake_secret)"

setup
printf 'see %s\n' "$TERM_TEXT" >later.txt && git add later.txt
git commit -q --no-verify -m "bypass the commit hooks"
expect refuse "pre-push refuses a private term committed with --no-verify" \
    git push -q origin HEAD:main

setup
printf 'token = "%s"\n' "$(fake_secret)" >later.txt && git add later.txt
git commit -q --no-verify -m "bypass the commit hooks"
expect refuse "pre-push refuses a secret on a new branch" git push -q origin HEAD:feature

setup
rm .private-terms
printf 'hello\n' >clean.txt && git add clean.txt
expect accept "without .private-terms a clean commit passes" git commit -q -m "add file"
if grep -q "no .private-terms file" "$WORK/out"; then
    ok "without .private-terms the hook warns"
else
    not_ok "without .private-terms the hook warns"
fi

# A linked worktree (git worktree add): its .git is a file pointing into the main
# repository, and it has no .private-terms of its own.
setup
rm -rf "$WORK/linked"
git worktree add -q -b linked "$WORK/linked"
cd "$WORK/linked" || exit 1
printf 'hello\n' >clean.txt && git add clean.txt
expect accept "in a linked worktree, a clean commit passes" git commit -q -m "add a clean file"
printf 'see %s\n' "$TERM_TEXT" >linked.txt && git add linked.txt
expect refuse "in a linked worktree, a private term is refused" git commit -q -m "add a file"
if grep -q "private term in the staged changes" "$WORK/out"; then
    ok "in a linked worktree, the main worktree's .private-terms apply"
else
    not_ok "in a linked worktree, the main worktree's .private-terms apply"
fi

printf '\n%s passed, %s failed\n' "$PASSED" "$FAILED"
[ "$FAILED" -eq 0 ]
