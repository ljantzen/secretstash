# export / import

## `stash export`

```
stash export [-o/--output <FILE>] [--include-history]
```

Exports all vault items to JSON. Output goes to stdout by default so it can be piped or redirected; use `-o`/`--output` to write directly to a file.

| Option | Description |
|--------|-------------|
| `-o`, `--output <FILE>` | Write to this file instead of stdout |
| `--include-history` | Include full version history for each item |

The JSON format is versioned (`"version": 1`) and includes the shortname, type, content, title, tags, browser preference, private flag, and timestamps for every item. History entries include their title at the time of archiving.

```sh
stash export -o vault.json
stash export --include-history -o vault-with-history.json
```

## `stash import`

```
stash import [FILE] [--overwrite]
```

Imports items from a JSON export file. Reads from stdin if `FILE` is omitted, making it composable with `stash export` via a pipe.

| Option | Description |
|--------|-------------|
| `FILE` | Path to the export file (omit to read from stdin) |
| `--overwrite` | Replace existing items instead of skipping them |

By default, items whose shortname already exists in the vault are skipped and counted in the summary. Pass `--overwrite` to delete the existing item (and its history) and replace it with the imported version. History entries present in the export file are restored in either case.

Prints a summary on completion: items imported, items skipped, items that failed (e.g. unknown type).

```sh
stash import vault.json
stash import --overwrite vault.json

# Pipe between vaults (e.g. copy to a different database)
stash export --include-history | stash import --db /path/to/other.db
```
