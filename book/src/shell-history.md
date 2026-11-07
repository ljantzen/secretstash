# Keeping Secrets Out of Shell History

When you pass content inline (`stash add note pw "s3cr3t"`), that content lands in your shell history. The cleanest remedies are to use `--edit` or `--stdin` instead; if you still want inline content, you can suppress history recording at the shell level.

## bash

Add to `~/.bashrc`:

```bash
# Ignore any command that starts with "stash"
HISTIGNORE="stash *:stash:$HISTIGNORE"
```

Alternatively, enable space-prefix suppression and lead every sensitive stash command with a space:

```bash
HISTCONTROL=ignorespace   # or ignoreboth (combines with ignoredups)
#  ↓ leading space — not recorded
 stash add note pw "s3cr3t"
```

## zsh

Add to `~/.zshrc`:

```zsh
# Option 1: ignore stash via a hook (works in all zsh versions)
zshaddhistory() { [[ $1 != stash\ * && $1 != stash$'\n' ]] }

# Option 2: space-prefix suppression (simpler, same caveat as bash)
setopt HIST_IGNORE_SPACE
#  ↓ leading space — not recorded
 stash add note pw "s3cr3t"
```

## fish

Fish has no pattern-based ignore list. Use `--stdin` or `--edit` to keep secrets out of the command line entirely. If you do pass inline content and want to remove the entry afterward:

```fish
history delete -- "stash add note pw s3cr3t"
```
