#!/usr/bin/env python3
"""
Recursively find git repositories under a path and run `git clean -dxf` in each.

Default is a dry-run that lists repositories. Use `--apply` (or `-a`) to actually
run the destructive command.

Usage examples:
  # dry-run current directory
  ./scripts/gitcleaner.py

  # dry-run specific directory
  ./scripts/gitcleaner.py /path/to/start

  # actually run git clean in each repo (destructive)
  ./scripts/gitcleaner.py --apply /path/to/start
"""
import argparse
import subprocess
import sys
from pathlib import Path


def find_git_repos(start: Path):
    """Return a sorted list of unique repo root Paths found under start."""
    repos = set()

    # Look for any entry named '.git' (could be a dir or file for worktrees/submodules)
    for p in start.rglob('.git'):
        try:
            parent = p.parent.resolve()
        except Exception:
            parent = p.parent
        repos.add(parent)

    return sorted(repos)


def run_git_clean(repo: Path) -> int:
    """Run `git clean -dxf` in repo. Return subprocess returncode."""
    try:
        res = subprocess.run(['git', 'clean', '-dxf'], cwd=str(repo))
        return res.returncode
    except FileNotFoundError:
        print("git not found on PATH", file=sys.stderr)
        return 127
    except Exception as e:
        print(f"Failed to run git in {repo}: {e}", file=sys.stderr)
        return 1


def main(argv=None):
    parser = argparse.ArgumentParser(description='Recursively run `git clean -dxf` in repositories.')
    parser.add_argument('start', nargs='?', default='.', help='Start path to scan (default: current dir)')
    parser.add_argument('-a', '--apply', action='store_true', help='Actually run `git clean -dxf` (destructive).')

    args = parser.parse_args(argv)
    start = Path(args.start).expanduser().resolve()

    if not start.exists():
        print(f"Start path does not exist: {start}", file=sys.stderr)
        return 2

    print(f"Scanning '{start}' for git repositories...")
    repos = find_git_repos(start)

    if not repos:
        print(f"No git repositories found under '{start}'.")
        return 0

    failed = []
    for repo in repos:
        print(f"Found repo: {repo}")
        if not args.apply:
            print(f"DRY-RUN: would run `git clean -dxf` in: {repo}")
            continue

        print(f"Running `git clean -dxf` in: {repo}")
        rc = run_git_clean(repo)
        if rc == 0:
            print(f"Succeeded: {repo}")
        else:
            print(f"Failed (code {rc}): {repo}", file=sys.stderr)
            failed.append((repo, rc))

    if failed:
        print(f"Finished with failures for {len(failed)} repos.", file=sys.stderr)
        return 1

    print("Done.")
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
