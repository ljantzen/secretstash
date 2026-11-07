# show

```
stash show <shortname> [-v/--verbose] [-c/--copy] [--clear-after <SECONDS>]
```

Prints the item's content. If the item has a title it is printed as a header line above the content. If the item has tags they are shown on a `tags:` line after the content. Add `--verbose` / `-v` to show all metadata (type, title, browser preference, private flag, timestamps, and tags) in a header block.

Add `--copy` / `-c` to copy the content to the clipboard instead of printing it (requires `pbcopy` on macOS, `wl-copy` on Wayland, `xclip` or `xsel` on X11, or `clip.exe` on Windows/WSL).

Pass `--clear-after <SECONDS>` to automatically clear the clipboard after the given number of seconds. This overrides `clipboard_clear_seconds` in `stash.toml` for a single invocation:

```sh
stash show mypassword --copy                  # copy; no automatic clear
stash show mypassword --copy --clear-after 30  # copy; clear after 30 s
```

Clear timeout resolution order: `--clear-after` flag > `clipboard_clear_seconds` in `stash.toml` > 0 (disabled).

```sh
stash show gh                   # prints title (if set) then content
stash show gh --verbose         # content + metadata (type, title, timestamps, tags)
stash show gh --copy            # copy content to clipboard (no terminal output)
```
