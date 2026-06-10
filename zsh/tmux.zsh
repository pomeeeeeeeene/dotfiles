# tmux with no arguments opens a session scoped to the current directory.
# tmux with one non-command argument opens that named session.
tmux() {
  if [[ $# -eq 0 ]]; then
    _tmux_open_session "$(basename "$PWD")"
    return
  fi

  if (( $# == 1 )) && [[ "$1" != -* ]] && ! _tmux_is_command "$1"; then
    _tmux_open_session "$1"
    return
  fi

  command tmux "$@"
}

_tmux_open_session() {
  local session
  session="$1"
  session="${session//[^[:alnum:]_.-]/_}"
  [[ -n "$session" ]] || session="root"

  if [[ -n "$TMUX" ]]; then
    command tmux has-session -t "=${session}" 2>/dev/null \
      || command tmux new-session -d -s "$session" -c "$PWD"
    command tmux switch-client -t "=${session}"
  else
    command tmux new-session -A -s "$session" -c "$PWD"
  fi
}

_tmux_is_command() {
  command tmux list-commands "$1" >/dev/null 2>&1
}
