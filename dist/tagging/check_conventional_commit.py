#!/usr/bin/env python3

from subprocess import Popen, PIPE
from typing import List
from sys import argv, stderr
import re


CONVENTIONAL_COMMIT_RE = re.compile(r"(feat|fix|chore)(\([\w_\-\/ ]+\))?: .+")


def eprint(*args, **kwargs):
    print(*args, file=stderr, **kwargs)


def cmd(args: List[str]) -> List[str]:
    proc = Popen(args, stdout=PIPE)
    (stdout, _) = proc.communicate()
    retcode = proc.returncode
    if retcode != 0:
        raise ValueError(f"Command {" ".join(args)} failed with code {retcode}")
    return stdout.decode().splitlines()


if __name__ == "__main__":
    target_branch = argv[1]
    if target_branch is None:
        eprint(f"Usage: {argv[0]} CURRENT_COMMIT TARGET_BRANCH")
        exit(1)
    cmd(["git", "fetch", "origin", f"{target_branch}:{target_branch}"])
    cmsgs = cmd(["git", "show", "-s", "--format=%s", f"{target_branch}..HEAD"])
    success = True
    for cmsg in cmsgs:
        if CONVENTIONAL_COMMIT_RE.match(cmsg) is None:
            eprint(f"Error: commit message '{cmsg}' does not follow the conventional commit standard")
    if not success:
        exit(1)
