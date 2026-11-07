# Quick Start

```sh
# First run: creates a new vault and prompts for a master password
stash auth login

# Login with a password from stdin (useful in scripts)
echo "mypassword" | stash auth login

# Login with a custom session timeout (overrides config for this session)
stash auth login --timeout 60   # expire after 60 minutes of inactivity
stash auth login --timeout 0    # never expire

# Add items: stash add <type> <name> [content]
stash add note todo  "Buy milk"
stash add url  gh    "https://github.com"
stash add url  work  "https://example.com" --browser firefox
stash add url  priv  "https://example.com" --private       # always open in private mode
stash add url  combo "https://example.com" -b firefox -p   # browser + private at creation

# Add an item with a title
stash add url gh "https://github.com" --title "GitHub"

# Add items with tags
stash add note todo "Buy milk" --tag work --tag personal

# Compose a longer note in your editor
stash add note journal --edit

# Read from stdin
echo "some text" | stash add note pipe --stdin

# Show an item
stash show gh                   # prints title (if set) then content
stash show gh --verbose         # content + metadata (type, title, timestamps, tags)
stash show gh --copy            # copy content to clipboard (no terminal output)

# Edit an item (opens $EDITOR; old version is archived automatically)
stash edit todo

# Update just the title, without opening an editor
stash edit gh --title "GitHub Home"

# View version history (includes title per version)
stash history gh

# Open a URL in the browser
stash web gh
stash web gh --private          # incognito / private mode
stash web gh --browser firefox  # use a specific browser (overrides config)

# Set stored browser preference for one or more URL items
stash browser firefox gh                    # set browser for one item
stash browser firefox gh work archive       # set browser for multiple items at once
stash browser firefox --all                 # set browser for every URL in the vault
stash browser firefox gh --clear            # clear stored browser preference
stash browser firefox gh -p                 # always open in private mode (-a / --private)
stash browser firefox gh --no-private       # clear private-mode preference

# List all items (TITLE column shown when any item has a title)
stash list

# List items filtered by tag (OR logic — shows items with any matching tag)
stash list --tag work
stash list --tag work --tag personal

# List items filtered by type
stash list --type url

# Manage tags on existing items
stash tag todo urgent
stash untag todo personal

# List all tags in the vault with item counts
stash tags

# Search item content and titles (case-insensitive substring)
stash search "github"

# Search using a regular expression (case-sensitive)
stash search --regex '\d{4}-\d{2}-\d{2}'

# Case-insensitive regex (use (?i) inline flag)
stash search --regex '(?i)github'

# Also search archived versions (shown as name:vN)
stash search --include-history "old value"
stash search --regex --include-history '(?i)todo'

# Filter by tag (repeatable; OR logic — shows items with any matching tag)
stash search --tag work
stash search --tag work --tag personal

# Filter by type
stash search --type url

# Combine: pattern + tag + type
stash search --tag work --type url "meeting"

# Delete an item and all its history
stash purge todo
stash purge todo --force        # skip confirmation (useful in scripts)

# Rename an item
stash rename todo todo-done

# Copy an item to a new shortname (copies content, title, type, and tags)
stash copy aws-key aws-key-backup

# Restore an item to its previous version (undo last edit; restores title too)
stash restore todo

# Restore to a specific version
stash restore todo --version 2

# End the session
stash auth logout

# Change the master password (re-keys the database in place)
stash auth reset

# Migrate an existing vault from the old field-level-encrypted format
stash migrate

# Export the entire vault to a JSON file
stash export -o vault.json

# Export including full version history for each item
stash export --include-history -o vault-with-history.json

# Import from a JSON export (skips items that already exist)
stash import vault.json

# Import and overwrite items that already exist
stash import --overwrite vault.json

# Pipe between vaults (e.g. copy to a different database)
stash export --include-history | stash import --db /path/to/other.db
```

Every command used above is documented in detail in the [Command Reference](./commands/index.md).
