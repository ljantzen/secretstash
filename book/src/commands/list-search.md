# list / search

## `stash list`

```
stash list [-g/--tag <TAG>]... [-t/--type <TYPE>]
```

Lists all items in a table showing name, type, and tags. When at least one item has a title, a TITLE column is added to the table. Pass one or more `-g`/`--tag` options to show only items that have **any** of the specified tags (OR logic). Pass `-t`/`--type` to restrict to a single type (`url` or `note`). Both filters can be combined.

```sh
stash list
stash list --tag work
stash list --tag work --tag personal
stash list --type url
```

## `stash search`

```
stash search [PATTERN] [--regex] [--include-history] [--tag <TAG>]... [--type <TYPE>]
```

Searches items by content, title, tag, and/or type. At least one of `PATTERN`, `--tag`, or `--type` is required.

| Option | Description |
|--------|-------------|
| `PATTERN` | Text or regex to match against content and titles |
| `-r`, `--regex` | Treat `PATTERN` as a regular expression |
| `-H`, `--include-history` | Also search archived versions (shown as `name:vN`) |
| `-g`, `--tag <TAG>` | Restrict to items with this tag (repeatable; OR logic) |
| `-t`, `--type <TYPE>` | Restrict results to `url` or `note` |

Without `--regex`, matching is a case-insensitive substring search. With `--regex`, the pattern is case-sensitive; prefix with `(?i)` for case-insensitive regex.

```sh
stash search "api key"
stash search --regex '\d{4}-\d{2}-\d{2}'
stash search --regex '(?i)secret'
stash search --tag work
stash search --tag work --tag personal
stash search --type url "github"
stash search --include-history "old password"
stash search --regex --include-history '(?i)todo'
```
