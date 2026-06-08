# dotfiles

Personal configuration files.

## Kakoune

This repository uses a GNU Stow compatible layout:

```sh
~/dotfiles/kak/.config/kak/kakrc
```

Apply manually:

```sh
ln -s ~/dotfiles/kak/.config/kak/kakrc ~/.config/kak/kakrc
```

Or with GNU Stow:

```sh
cd ~/dotfiles
stow kak
```

Dependencies:

- kakoune
- kak-lsp
- clangd
- ripgrep
- pbcopy/pbpaste on macOS
