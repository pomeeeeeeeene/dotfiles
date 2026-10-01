{ pkgs }:

pkgs.writeShellScriptBin "apply-gnome-key-map" ''
  set -eu

  schema="org.gnome.desktop.input-sources"
  key="xkb-options"

  current="$(pkgs.glib}/bin/gsettings get "$schema" "$key")"

  if [ "$current" != "@as []" ] && [ "$current" != "[]" ]; then
    echo "Existing GNOME xkb-options found:"
    echo "$current"
    echo "Refusing to overwrite existing keymap settings."
    exit 1
  fi

  ${pkgs.glib}/bin/gsettings set \
    "$schema" \
    "$key" \
    "['ctrl:nocaps']"
  ''
