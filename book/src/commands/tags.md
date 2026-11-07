# tag / untag / tags

## `stash tag <shortname> <TAG>...`

Adds one or more tags to an existing item. Duplicate tags are silently ignored.

```sh
stash tag todo urgent
```

## `stash untag <shortname> <TAG>...`

Removes one or more tags from an existing item. Tags not present on the item are silently ignored.

```sh
stash untag todo personal
```

## `stash tags`

Lists every tag in the vault, sorted alphabetically, with a count of how many items carry each tag:

```
TAG        ITEMS
──────────────────
personal   3
work       5

2 tag(s).
```
