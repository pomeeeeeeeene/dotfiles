# dotfiles

個人用の設定ファイルです。

## Kakoune

このリポジトリは GNU Stow 互換のレイアウトにしています。

```sh
~/dotfiles/kak/.config/kak/kakrc
```

手動で適用する場合:

```sh
ln -s ~/dotfiles/kak/.config/kak/kakrc ~/.config/kak/kakrc
```

GNU Stow を使う場合:

```sh
cd ~/dotfiles
stow kak
```

依存コマンド:

- kakoune
- kak-lsp
- clangd
- ripgrep
- bear
- pbcopy/pbpaste on macOS

### C/C++ LSP

C/C++ で LSP を正しく使うには、プロジェクトに `compile_commands.json` があるとよいです。
`compile_commands.json` は `clangd` にコンパイルオプション、include path、macro 定義などを伝えるためのファイルです。

CMake プロジェクトでは、次のように生成できます。

```sh
cmake -S . -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
ln -s build/compile_commands.json .
```

Makefile プロジェクトでは、`bear` を使って `make` の実行内容から `compile_commands.json` を生成できます。

```sh
bear -- make
```

すでにビルド済みでコンパイルが走らない場合は、先に clean してから実行します。

```sh
make clean
bear -- make
```

`.o` ファイルなどのビルド成果物は LSP には不要なので、Git に含める必要はありません。
