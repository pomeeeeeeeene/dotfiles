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

### C/C++ LSP

C/C++ で LSP を正しく使うには、プロジェクトに `compile_commands.json` があるとよいです。
`compile_commands.json` は `clangd` にコンパイルオプション、include path、macro 定義などを伝えるためのファイルです。

CMake プロジェクトでは、次のように生成できます。

```sh
cmake -S . -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
ln -s build/compile_commands.json .
```

`.o` ファイルなどのビルド成果物は LSP には不要なので、Git に含める必要はありません。
