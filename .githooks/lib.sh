# Shared code of the git hooks (ADR-0041). POSIX sh; needs git, grep, awk, and either
# a local Betterleaks of the pinned version or Docker.

BETTERLEAKS_VERSION=1.10.1
BETTERLEAKS_IMAGE=ghcr.io/betterleaks/betterleaks@sha256:f345c1ac502abc51ec334c3d815ad18a79fb1509555fc4496b2cb88e760681b3

ROOT=$(git rev-parse --show-toplevel) || exit 1
# The git directory all worktrees share. In a linked worktree (`git worktree add`) it
# lies outside ROOT, and the main worktree next to it holds .private-terms.
COMMON=$(git rev-parse --path-format=absolute --git-common-dir) || exit 1
TMP=$(mktemp -d) || exit 1
trap 'rm -rf "$TMP"' EXIT
FAILED=0

fail() {
    printf 'iasion hook: %s\n' "$1" >&2
    FAILED=1
}

finish() {
    if [ "$FAILED" -ne 0 ]; then
        printf 'iasion hook: refused. Fix the findings above (ADR-0039, ADR-0041).\n' >&2
        exit 1
    fi
    exit 0
}

# --- Betterleaks ---------------------------------------------------------------------

# Runs Betterleaks with the given arguments, locally if the pinned version is installed,
# otherwise in its container image pinned by digest. Paths inside the repository must be
# given relative to its root ("." is the root). Reads stdin.
betterleaks_run() {
    if command -v betterleaks >/dev/null 2>&1 &&
        [ "$(betterleaks version 2>/dev/null)" = "$BETTERLEAKS_VERSION" ]; then
        (cd "$ROOT" && betterleaks "$@" --redact --no-banner --log-level warn)
    elif command -v docker >/dev/null 2>&1; then
        # The shared git directory at its own path too: a linked worktree's .git is a
        # file that points into it by absolute path.
        docker run --rm -i --user "$(id -u):$(id -g)" -v "$ROOT:/repo" -v "$COMMON:$COMMON" -w /repo \
            "$BETTERLEAKS_IMAGE" "$@" --redact --no-banner --log-level warn
    else
        printf 'iasion hook: needs betterleaks %s or Docker to scan for secrets.\n' \
            "$BETTERLEAKS_VERSION" >&2
        return 2
    fi
}

# --- Private terms -------------------------------------------------------------------

# Prepares the terms in "$TMP/terms": no empty lines, no comments, no carriage returns.
# Returns 1, with a warning, if there are none.
terms_load() {
    file="$ROOT/.private-terms"
    # A linked worktree has none of its own; the main worktree's applies.
    if [ ! -f "$file" ]; then
        file="$(dirname "$COMMON")/.private-terms"
    fi
    if [ ! -f "$file" ]; then
        printf 'iasion hook: no .private-terms file; private terms are not checked.\n' >&2
        return 1
    fi
    tr -d '\r' <"$file" | grep -v -e '^[[:space:]]*$' -e '^[[:space:]]*#' >"$TMP/terms"
    if [ ! -s "$TMP/terms" ]; then
        printf 'iasion hook: .private-terms has no terms; nothing to check.\n' >&2
        return 1
    fi
    return 0
}

# True if stdin contains a private term. Matched as fixed strings, ignoring case.
has_term() {
    grep -q -F -i -f "$TMP/terms"
}

# Prints a path, or a placeholder if the path itself contains a private term, so that
# no term is ever printed.
safe_path() {
    if printf '%s\n' "$1" | has_term; then
        printf '(a file whose path contains a private term)'
    else
        printf '%s' "$1"
    fi
}

# Reads a unified diff with -U0 on stdin and prints "path<TAB>line<TAB>content" for every
# added line.
added_lines() {
    awk '
        /^\+\+\+ / { path = substr($0, 7); next }
        /^@@/ { match($0, /\+[0-9]+/); line = substr($0, RSTART + 1, RLENGTH - 1) + 0; next }
        /^\+/ { print path "\t" line "\t" substr($0, 2); line++ }
    '
}

# Reports every line of the file $2 ("path<TAB>line<TAB>content") whose content holds a
# private term, by path and line number only. $1 describes where the lines are from.
report_lines() {
    cut -f3- "$2" | grep -n -F -i -f "$TMP/terms" | cut -d: -f1 >"$TMP/hits"
    while IFS= read -r n; do
        path=$(sed -n "${n}p" "$2" | cut -f1)
        line=$(sed -n "${n}p" "$2" | cut -f2)
        fail "private term in $1: $(safe_path "$path"), line $line"
    done <"$TMP/hits"
}

# Reports how many paths in the file $2 (one per line) hold a private term, without
# printing them. $1 describes where the paths are from.
report_paths() {
    count=$(grep -c -F -i -f "$TMP/terms" "$2")
    if [ "$count" -gt 0 ]; then
        fail "private term in the path of $count file(s) in $1"
    fi
}

# Reports private terms in the message in file $2, by line number. $1 describes it.
report_message() {
    grep -n -F -i -f "$TMP/terms" "$2" | cut -d: -f1 >"$TMP/hits"
    while IFS= read -r n; do
        fail "private term in $1, line $n"
    done <"$TMP/hits"
}

# Used by the hooks that source this file.
# shellcheck disable=SC2034
DIFF_OPTIONS="-U0 --no-color --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/"
