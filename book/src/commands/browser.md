# browser

Sets or clears browser preferences stored on one or more `url`-type items.

```
stash browser <BROWSER> <URL>... [options]
stash browser <BROWSER> --all    [options]
```

| Argument / Option | Description |
|-------------------|-------------|
| `<BROWSER>` | Browser binary to store (e.g. `firefox`, `chromium`) |
| `<URL>...` | One or more URL item shortnames to update |
| `--all` | Apply to every URL item in the vault (conflicts with named shortnames) |
| `--clear` | Remove the stored browser preference (`<BROWSER>` is ignored) |
| `-a`, `--private` | Always open this URL in private/incognito mode |
| `-n`, `--no-private` | Clear the stored private-mode preference |

Flags can be combined: e.g. `stash browser firefox gh -a` sets both browser and private mode in one command. `--private` and `--no-private` are mutually exclusive.

```sh
stash browser firefox gh                  # set stored browser for one item
stash browser firefox gh work archive     # set browser for several items at once
stash browser firefox --all               # set browser for every URL in the vault
stash browser firefox gh --clear          # remove stored browser preference
stash browser firefox gh -a               # always open in private mode
stash browser firefox gh --no-private     # clear private-mode preference
stash browser firefox gh -a --clear       # clear browser pref, set private mode
```

See [web](./web.md) for how these stored preferences interact with `-p`/`--browser` flags and `stash.toml` defaults.
