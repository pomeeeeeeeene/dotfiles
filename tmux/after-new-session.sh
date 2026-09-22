#!/bin/sh
set -eu

socket_path=$1

session_id=$(
  tmux -S "$socket_path" list-sessions -F '#{session_created} #{session_id}' \
    | sort -n \
    | tail -1 \
    | awk '{print $2}'
)

bottom_pane=$(
  tmux -S "$socket_path" list-panes -t "$session_id:" -F '#{pane_id}' \
    | head -1
)

pane_path=$(
  tmux -S "$socket_path" display-message -p -t "$bottom_pane" '#{pane_current_path}'
)
left_top_pane=$(
  tmux -S "$socket_path" split-window -v -b -l 70% -c "$pane_path" -t "$bottom_pane" -P -F '#{pane_id}'
)
tmux -S "$socket_path" split-window -h -p 50 -c "$pane_path" -t "$left_top_pane"

tmux -S "$socket_path" select-pane -t "$bottom_pane"
