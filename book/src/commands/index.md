# Command Reference

Every `stash` invocation accepts a global `--db <FILE>` flag (before or after the subcommand) that overrides the database path for that single invocation — see [Configuration](../configuration.md) for the full resolution order.

The subcommands are grouped below by what they operate on:

| Group | Commands |
|-------|----------|
| [Authentication](./auth.md) | `auth login`, `auth status`, `auth logout`, `auth reset` |
| [Adding items](./add.md) | `add` |
| [Reading items](./show.md) | `show` |
| [Editing items](./edit.md) | `edit` |
| [History](./history.md) | `history`, `restore` |
| [Browser](./web.md) | `web` |
| [Browser preferences](./browser.md) | `browser` |
| [Listing & searching](./list-search.md) | `list`, `search` |
| [Tags](./tags.md) | `tag`, `untag`, `tags` |
| [Item management](./item-management.md) | `rename`, `copy`, `purge` |
| [Export & import](./export-import.md) | `export`, `import` |
| [Migration](./migrate.md) | `migrate` |
| [Shell completions](./completions.md) | `completions` |
