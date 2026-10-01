{ pkgs }:

let
  xkbData = pkgs.runCommand "dotfiles-gnome-xkb" {} ''
    mkdir -p $out/rules $out/symbols
    cp ${./xkb/rules/evdev} $out/rules/evdev
    cp ${./xkb/symbols/dotfiles} $out/symbols/dotfiles
  '';

  apply = pkgs.writeShellScriptBin "apply-gnome-key-map" ''
    set -eu

    schema="org.gnome.desktop.input-sources"
    key="xkb-options"
    gsettings="$(command -v gsettings || true)"
    install="${pkgs.coreutils}/bin/install"
    cmp="${pkgs.coreutils}/bin/cmp"

    if [ -z "$gsettings" ]; then
      echo "gsettings was not found in PATH."
      exit 1
    fi

    current="$($gsettings get "$schema" "$key")"
    desired="['ctrl:nocaps', 'custom:dotfiles']"

    case "$current" in
      "@as []"|"[]"|"['ctrl:nocaps']"|"['ctrl:nocaps', 'custom:dotfiles']"|"['custom:dotfiles', 'ctrl:nocaps']")
        ;;
      *)
        echo "Existing GNOME xkb-options found:"
        echo "$current"
        echo "Refusing to overwrite existing keymap settings."
        exit 1
        ;;
    esac

    xkb_dir="''${XDG_CONFIG_HOME:-$HOME/.config}/xkb"

    install_xkb_file() {
      source="$1"
      destination="$2"

      if [ -e "$destination" ] || [ -L "$destination" ]; then
        if ! "$cmp" "$source" "$destination" >/dev/null; then
          echo "Existing XKB file differs: $destination"
          echo "Refusing to overwrite existing XKB configuration."
          exit 1
        fi
        return
      fi

      "$install" -D -m 0644 "$source" "$destination"
    }

    install_xkb_file "${xkbData}/rules/evdev" "$xkb_dir/rules/evdev"
    install_xkb_file "${xkbData}/symbols/dotfiles" "$xkb_dir/symbols/dotfiles"

    "$gsettings" set \
      "$schema" \
      "$key" \
      "$desired"

    echo "Applied GNOME keymap: ctrl:nocaps, custom:dotfiles"
  '';
in

pkgs.symlinkJoin {
  name = "keymap-gnome";
  paths = [ apply xkbData ];
}
