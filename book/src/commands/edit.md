# edit

```
stash edit <shortname> [-t/--title <TITLE>]
```

Without `--title`, opens the item in `$EDITOR`. If the content changes, the previous version is automatically saved to history before the update is written.

With `--title <TITLE>`, updates the title only without opening an editor. The current content and old title are archived to history as part of the update.

```sh
stash edit todo                       # opens $EDITOR
stash edit gh --title "GitHub Home"   # title-only update
```

See also [history / restore](./history.md) for reviewing and undoing edits.
