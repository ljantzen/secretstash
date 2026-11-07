# history / restore

## `stash history <shortname>`

Shows all archived versions followed by the current content, each labelled with its version number and timestamp. If a version had a title set, it is shown alongside the content for that version.

```sh
stash history gh
```

## `stash restore <shortname>`

```
stash restore <shortname> [--version <N>]
```

Restores an item to a previous version. Without `--version`, restores to the most recently archived version (undo last edit). Both content and title are restored from the history entry. The current content and title are archived before the restore, so the full history is preserved.

```sh
stash restore todo               # undo last edit
stash restore todo --version 2   # restore a specific version
```
