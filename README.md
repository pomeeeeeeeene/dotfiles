# dotfiles

GNU Stow で管理する個人用 dotfiles です。

## パッケージ構成

```text
kak/   Kakoune 設定
yazi/  Kakoune プレビュー連携用の Yazi 設定
yazi-cli-notes/  cli-notes tmux session 用の Yazi override
bin/   ~/.local/bin に置く補助スクリプト
tmux/  tmux 設定
```

## インストール

```sh
git clone <repo-url> ~/dotfiles
cd ~/dotfiles
stow kak yazi yazi-cli-notes bin tmux
```

リンク先のファイルが既に存在する場合は、事前に退避するか、このリポジトリへ取り込んでください。

## 依存コマンド

- GNU Stow
- Kakoune
- kak-lsp
- clangd
- Yazi
- tmux
- jq
- ripgrep
- macOS の `pbcopy` / `pbpaste`
- tmux の plugin 管理を使う場合は TPM

## Kakoune

Kakoune 設定では主に次を有効にしています。

- C/C++/Objective-C 向けの `kak-lsp` + `clangd`
- C/C++ の semantic token highlight
- C の定義ジャンプ、参照検索、hover 補助コマンド
- macOS クリップボード連携
- 任意の 42 header 挿入コマンド

個人情報や環境依存の override は Git 管理しません。必要なら次のファイルに置きます。

```sh
~/.config/kak/local.kak
```

例:

```kak
declare-option str ft42_header_name yourname
declare-option str ft42_header_email yourname@example.com
```

## Yazi + Kakoune プレビュー

Yazi の移動キーから次のスクリプトを呼びます。

```sh
~/.local/bin/kak-preview-sync
```

このスクリプトは tmux の現在 window 内にある Kakoune セッションを探し、Yazi で選択中のファイルを Kakoune 側に表示します。C/C++ ファイルでは LSP の semantic highlight も更新します。

C/C++ 用には、生成した `compile_commands.json` を次の cache 配下に置きます。

```sh
~/.cache/kak-preview-sync/clangd/
```

プロジェクト直下の `compile_commands.json` は書き換えません。既に存在する場合はそれを元にしつつ、未登録の `.c` ファイルだけ cache 側で補完します。

Yazi で高速に移動したときに古い clangd request が溜まらないよう、semantic token 更新は debounce しています。待ち時間は次で調整できます。

```sh
export KAK_PREVIEW_SYNC_DEBOUNCE=0.18
```

必要に応じて次の環境変数で挙動を上書きできます。

```sh
export KAK_SESSION=auto        # または明示的な Kakoune session 名
export KAK_CLIENT=client0      # 対象 Kakoune client
export KAK_SYNC_SCOPE=my-scope # state/log の名前空間
```

ログと状態ファイルは `${TMPDIR:-/tmp}` 配下に置きます。

## Session wrappers

`bin/` には tmux session 単位で Kakoune/Yazi の状態を揃える wrapper も含めています。

```sh
~/.local/bin/yazi-session
~/.local/bin/yazi-session-core
~/.local/bin/kak-session
```

`yazi-session` は `yazi-session-core` を探して実行する薄い wrapper です。`yazi-session-core` は tmux session 名から次を設定します。

- `KAK_SESSION`
- `KAK_SYNC_SCOPE`
- `XDG_RUNTIME_DIR`
- `YAZI_CONFIG_HOME`

`~/.config/yazi-<tmux-session>/` が存在する場合は、通常の `~/.config/yazi/` に session-specific な設定を重ねます。例として `yazi-cli-notes/` パッケージでは `cli-notes` session 用に Yazi の pane ratio だけを上書きしています。

また、リポジトリ root に `yazi.toml` がある場合は、tmux 内ではその設定を現在の Yazi 設定へ merge できます。無効化する場合は次を設定します。

```sh
export YAZI_DIR_CONFIG=false
```

## tmux

tmux 設定では、Yazi から Kakoune socket を安定して見つけられるように `TMPDIR` と `XDG_RUNTIME_DIR` を tmux 環境へ引き継ぎます。

また TPM plugin を参照しています。plugin 管理を使う場合は TPM を別途入れてください。

```sh
git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm
```

その後 tmux 設定を reload し、TPM の install key binding を実行します。
