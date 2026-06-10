# tmux with no arguments opens a session scoped to the current directory.
tmux() {
  if [[ $# -ne 0 ]]; then
    command tmux "$@"
    return
  fi

  local session
  session="$(basename "$PWD")"
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
