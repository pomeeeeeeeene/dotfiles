#!/usr/bin/env zsh

SCRIPT_DIR="${0:A:h}"

source "$SCRIPT_DIR/options.zsh"

xkb_options="["
for option in "${XKB_OPTIONS[@]}"; do
  xkb_options+="'$option', "
done
xkb_options="${xkb_options%, }]"

gsettings set \
  org.gnome.desktop.input-sources \
  xkb-options \
  "$xkb_options"
