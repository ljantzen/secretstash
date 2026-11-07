# web

```
stash web [-p/--private] [-b/--browser <browser>] <shortname>
```

Opens a `url`-type item in the browser. Pass `-p` / `--private` to open in private/incognito mode. Pass `-b` / `--browser` to specify a browser binary (e.g., `firefox`, `google-chrome`); this overrides the `browser` field in `stash.toml`. Without a specified browser, the system default is used (or, for `--private`, tries Firefox, Chrome, Chromium, Brave, and Vivaldi in order).

If the item has a stored private-mode preference (set via `stash browser --private`), private mode is activated automatically even without `-p`.

**Browser resolution order:** `--browser` flag > per-item stored browser ([`stash browser`](./browser.md)) > `browser` in `stash.toml` > system default.

**Private mode resolution order:** `-p` flag > per-item stored private preference > `private` in `stash.toml` > off.

```sh
stash web gh
stash web gh --private          # incognito / private mode
stash web gh --browser firefox  # use a specific browser (overrides config)
```

## Browser name shorthand

Anywhere a browser name is accepted (`--browser`, `stash browser`, `stash add -b`), you can use the shortest unambiguous prefix of a known browser name instead of the full name:

```sh
stash web gh -b fi        # resolves to firefox
stash web gh -b go        # resolves to google-chrome
stash web gh -b brave     # resolves to brave-browser
stash web gh -b vi        # error: ambiguous (vivaldi, vivaldi-stable)
stash web gh -b vivaldi-  # resolves to vivaldi-stable
```

Unknown names (not in the built-in list) are passed through as-is, so custom browsers always use their exact binary name.

**Built-in private-mode support:** `firefox` (`--private-window`), `google-chrome` / `chrome`, `chromium`, `chromium-browser`, `brave-browser`, `vivaldi`, `vivaldi-stable` (`--incognito`). `chrome` is accepted as an alias for `google-chrome`.

**Custom browsers:** add a `[browser_flags]` section to `stash.toml` to enable private mode for any other browser:

```toml
[browser_flags]
opera      = "--private"
my-browser = "--incognito"
```
