// How the page shows which release runs: the first seven characters of the commit,
// as Git abbreviates it, or `dev` for a local build.
export function revisionLabel(revision: string): string {
  return /^[0-9a-f]{40}$/.test(revision) ? revision.slice(0, 7) : revision
}
