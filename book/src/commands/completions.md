# completions

```
stash completions <SHELL>
```

Prints a shell completion script to stdout. Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

Source the output in your shell's startup file to get tab-completion for all subcommands, flags, and arguments:

```sh
# bash — add to ~/.bashrc
source <(stash completions bash)

# zsh — add to ~/.zshrc
source <(stash completions zsh)

# fish — save to the completions directory
stash completions fish > ~/.config/fish/completions/stash.fish
```
