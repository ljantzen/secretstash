# add

```
stash add <url|note> <name> [TEXT] [options]
```

| Argument / Option | Description |
|-------------------|-------------|
| `url\|note` | Item type (required, positional) |
| `<name>` | Short identifier used in all other commands (required, positional) |
| `TEXT` | Inline content (optional positional) |
| `-t`, `--title <TITLE>` | Human-readable title |
| `-e`, `--edit` | Open `$EDITOR` to compose content |
| `--stdin` | Read content from standard input |
| `-g`, `--tag <TAG>` | Attach a tag (repeatable: `--tag work --tag personal`) |
| `-b`, `--browser <BROWSER>` | Store a preferred browser (url items only) |
| `-p`, `--private` | Always open this URL in private/incognito mode (url items only) |
| `-n`, `--no-private` | Clear the stored private-mode preference (url items only) |

Exactly one of `TEXT`, `--edit`, or `--stdin` must be supplied. `--private` and `--no-private` are mutually exclusive.

```sh
stash add note todo  "Buy milk"
stash add url  gh    "https://github.com"
stash add url  gh    "https://github.com" --title "GitHub"
stash add note todo  "Buy milk" --tag work --tag personal
stash add url  work  "https://example.com" --browser firefox
stash add url  priv  "https://example.com" --private
stash add url  combo "https://example.com" -b firefox -p
stash add note journal --edit
echo "some text" | stash add note pipe --stdin
```
