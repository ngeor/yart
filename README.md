# yart

yart is yet another release tool

## Overview

You can use yart to automatically bump up the version of a project in a
consistent way, ensuring no gaps exist in SemVer sequences.

## Workflow

- You have a git repository.
- You use git tags for versioning.
- Tags are in the format vMajor.Minor.Patch (e.g. v1.2.3).
- You work directly in the master branch.
- You have committed some new changes and want to create a new version.

If you run `yart -v minor`, it will:

- determine the current version from the latest git tag
- derive the desired version by incrementing the minor SemVer component
- update project files that reference the old version (see further down about Files) and commit them
- create the git tag
- push

## Motivation

The inspiration comes from `npm version minor` which does more or less the same, but for npm projects.

For Maven projects, consider `mvn release:prepare`.

## Installation

TODO

## Tags

yart reads and sorts the git tags of the repo to determine the current version,
which is the greatest.

No gaps in SemVer are allowed, so if the current version is 1.2.3, the allowed
values for the next version are:

- 1.2.4 (patch)
- 1.3.0 (minor)
- 2.0.0 (major)

yart will tag the version and push it to git in a consistent format:

- the tag will be prefixed with v, e.g. v1.3.0
- the tag message will be "Releasing version 1.3.0"

## Files

The version might be present in language specific files and yart tries to bump
the version there too.

### Readme files

yart will do a text replace in files named `README.md`, replacing the current
version with the new version.

### TODO support more stacks

[ ] VB6
[ ] Cargo

## Options

-  `-V, --version`        :  output the version number
-  `-v <version>`         :  The new version to use. Must be semver and not
   leave gaps from previous version. It can also be one of major, minor, patch
   to automatically increment to the next version.
-  `--dir [dir]`          :  The directory to run the command in (default: ".")
-  `--message [message]`  :  An optional commit message
-  `--dry-run`            :  Do not perform any changes, see what would happen
-  `--no-push`            :  Do not push
-  `--verbose`            :  Increase logging verbosity
-  `-h, --help`           : output usage information
