#!/usr/bin/env python3

from subprocess import Popen, PIPE
import os
import sys
from typing import List


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


if not os.path.isfile("Cargo.toml") or not os.path.isdir(".git"):
    eprint("This script needs to run from the root directory of the project")
    sys.exit(1)

WHATBUMP_SCRIPT = "./dist/tagging/whatbump.sh"

if not os.path.isfile(WHATBUMP_SCRIPT):
    eprint("whatbump script not found")
    sys.exit(1)


def cmd(args: List[str]):
    proc = Popen(args)
    proc.communicate()
    retcode = proc.returncode
    if retcode != 0:
        raise ValueError(f"Command {" ".join(args)} failed with code {retcode}")


def whatbump():
    proc = Popen(["bash", WHATBUMP_SCRIPT], stdout=PIPE)
    (stdout, _) = proc.communicate()
    retcode = proc.returncode
    if retcode != 0:
        raise ValueError(f"whatbump script terminated with code {retcode}")
    return stdout.decode().strip()


def update_meson_build(tag: str):
    with open("meson.build", "r") as fd:
        rows = fd.readlines()
    rowfound = False
    for i, row in enumerate(rows):
        if row.startswith("  version: '") and row.endswith("', # version number row\n"):
            rows[i] = f"  version: '{tag}', # version number row\n"
            rowfound = True
            break
    if not rowfound:
        eprint("Could not find version row in meson.build")
        sys.exit(1)
    with open("meson.build", "w") as fd:
        fd.writelines(rows)


def update_cargo_toml(tag: str):
    with open("Cargo.toml", "r") as fd:
        rows = fd.readlines()
    rowfound = False
    for i, row in enumerate(rows):
        if row.startswith('version = "') and row.endswith('"\n'):
            rows[i] = f'version = "{tag}"\n'
            rowfound = True
            break
    if not rowfound:
        eprint("Could not find version row in Cargo.toml")
        sys.exit(1)
    with open("Cargo.toml", "w") as fd:
        fd.writelines(rows)


CHANGELOG_PATH = "./changelog_metainfo.xml"
METAINFO_PATH = "./data/org.gabmus.envision.metainfo.xml.in.in"


def update_metainfo(tag: str):
    with open(CHANGELOG_PATH, "r") as fd:
        chlog = fd.read()
    with open(METAINFO_PATH, "r") as fd:
        rows = fd.readlines()
    for i, row in enumerate(rows):
        if "<releases>\n" in row:
            rows.insert(i + 1, chlog)
            break
    with open(METAINFO_PATH, "w") as fd:
        fd.write("".join(rows))
    os.unlink(CHANGELOG_PATH)


def yes_no() -> bool:
    print("Continue? (y/n)")
    print(">>> ", end="")
    res = input()
    return res[0].lower() == "y"


if __name__ == "__main__":
    tag = whatbump()
    print(f"New candidate tag: {tag}")
    if not yes_no():
        os.unlink(CHANGELOG_PATH)
        sys.exit(0)
    update_meson_build(tag)
    update_cargo_toml(tag)
    update_metainfo(tag)
    cmd(["git", "diff"])
    commitmsg = f"chore: update version to {tag}"
    print(f"Will commit with the following message: '{commitmsg}'")
    if not yes_no():
        sys.exit(0)
    cmd(["git", "add", "meson.build", "Cargo.toml", "Cargo.lock", METAINFO_PATH])
    cmd(["git", "commit", "-m", commitmsg])
    print(f"Will add tag '{tag}'")
    if not yes_no():
        sys.exit(0)
    cmd(["git", "tag", "-a", tag, "-m", ""])
    print(f"Will push new commit and tag to origin")
    if not yes_no():
        sys.exit(0)
    cmd(["git", "push"])
    cmd(["git", "push", "origin", tag])
