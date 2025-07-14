# Git from Scratch – Student Project

This project involves reimplementing core Git functionality in Rust. It will help you understand Git's internal architecture and make you comfortable with both its plumbing and porcelain commands.

##### Sub-repo:
[Homebrew Tap](https://github.com/Oomaxime/homebrew-guts)

#### Instalation:
https://github.com/Jeck0v/Guts/wiki/Installation-with-package-manager

## 🎯 Project Scope

You will implement a subset of Git commands, both low-level (plumbing) and user-facing (porcelain). Your goal is to ensure they behave similarly to real Git, within clearly defined constraints.

## How to use the TUI app:
You need to:
``` bash
cargo build
```
and then:
``` bash
cargo run -- tui
```
---

## 🛠 Plumbing Commands

### `git hash-object <file>` :white_check_mark:
- **Creates a blob** object from file content and writes its SHA-1 to stdout.
- ❌ Reject directories or missing files.

### `git cat-file -t|-p <oid>` :white_check_mark:
- `-t`: Print object type.
- `-p`: Pretty-print blob/tree/commit content.
- ❌ Reject invalid OIDs or missing options.

### `git write-tree` :white_check_mark:
- Create a tree object from the staging area.
- Writes SHA-1 of the tree to stdout.

### `git commit-tree <tree_sha> -m "msg" [-p <parent>]` :white_check_mark:
- Creates a commit object pointing to a tree (and parent commit if any) and writes its oid to stdout
- Requires `-m` message.
- ❌ No annotated tags.

---

## 🧑‍💻 Porcelain Commands

### `git init [<dir>]` :white_check_mark:
- Initializes a Git repository in the given directory.
- Create .git/objects, .git/refs/heads, HEAD, and minimal config

### `git add <file>…`
- Adds files to the staging area (not directories).
- ❌ No `-p`, no wildcards.

### `git rm <file>…`
- Removes a file from working directory and index.

### `git commit -m "msg"`
- Runs `write-tree`, creates a commit with HEAD as parent.
- ❌ No editor or message prompt.

### `git status`
- Shows staged and unstaged changes.

### `git checkout [-b] <branch|sha>`
- Switch to existing commit or branch.
- `-b <branch>` creates a new branch.
- Change HEAD, update working dir, check for conflicts

### `git reset [--soft|--mixed|--hard] <sha>`
- `--soft`: move HEAD
- `--mixed`: + reset index
- `--hard`: + reset working directory
- ❌ No file-specific reset.

### `git log`
- Print commit history from HEAD (one-line summary ok).

### `git ls-files`
- List all files in the index.

### `git ls-tree <tree_sha>`
- List contents of a tree object.

### `git rev-parse <ref>`
- Convert ref/branch/HEAD into SHA-1.
- ❌ No complex selectors

### `git show-ref`
- List all refs and their hashes.

---

## 🧠 Advanced Feature: Merge Support

### `git merge <branch|sha>`
- Perform 3-way merge and create a merge commit with 2 parents.
- On conflict: insert `<<<<<<<`, `=======`, `>>>>>>>` markers into file(s).
- ❌ No rebase, squash, or fast-forward-only merges.

---

## 📄 Gitignore

- Handle `.gitignore`
- Use simple glob-style matching (e.g., `*.log`, `build/`)
- ❌ No negation or nested `.gitignore` files.

---

## 🏗 Index Implementation

- You are free to implement the index your way.
- ✅ Bonus if it matches Git’s format closely.

---

## ❌ Out of Scope

- `git push` and `git update-index` are NOT required.
- No support for remotes, rebase, tags, or stashing.

---

## ✅ Deliverables

- A working implementation of the listed commands.
- Tests and example usage for each.
- Clean error handling for all unsupported cases.

---

## ⏱ Time Estimate

- **Total time**: 6–9 days
- Use AI if needed, but understand what you're coding.

---
