# rename / copy / purge

## `stash rename <shortname> <new-name>`

Renames an item. Fails if `<new-name>` is already in use.

```sh
stash rename todo todo-done
```

## `stash copy <shortname> <dest>`

Copies an item (content, title, type, and tags) to a new shortname. History is not copied.

```sh
stash copy aws-key aws-key-backup
```

## `stash purge <shortname>`

Deletes an item and its entire history after a confirmation prompt. Pass `--force` / `-f` to skip the prompt (useful in scripts).

```sh
stash purge todo
stash purge todo --force        # skip confirmation
```
