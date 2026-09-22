(( $+parameters[FAST_HIGHLIGHT_VERSION] )) && return 0
(( $+commands[brew] )) || return 0

_shared_fast_syntax_highlighting_prefix="$(brew --prefix zsh-fast-syntax-highlighting 2>/dev/null)" || {
  unset _shared_fast_syntax_highlighting_prefix
  return 0
}
_shared_fast_syntax_highlighting_plugin="${_shared_fast_syntax_highlighting_prefix}/share/zsh-fast-syntax-highlighting/fast-syntax-highlighting.plugin.zsh"

if [[ -r "$_shared_fast_syntax_highlighting_plugin" ]]; then
  source "$_shared_fast_syntax_highlighting_plugin"
fi

unset _shared_fast_syntax_highlighting_prefix _shared_fast_syntax_highlighting_plugin
