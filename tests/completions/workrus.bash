# bash completion for workrus
_workrus() {
  local cur
  cur="${COMP_WORDS[COMP_CWORD]}"
  case "${COMP_WORDS[1]}" in
    team) COMPREPLY=( $(compgen -W 'list id members create autolinks' -- "$cur") ) ;;
    project) COMPREPLY=( $(compgen -W 'list view create' -- "$cur") ) ;;
    milestone|m) COMPREPLY=( $(compgen -W 'list view create update delete' -- "$cur") ) ;;
    document|docs) COMPREPLY=( $(compgen -W 'list view create update delete' -- "$cur") ) ;;
    issue)
      if [[ ${COMP_WORDS[2]} == comment ]]; then COMPREPLY=( $(compgen -W 'list add update delete' -- "$cur") ); else COMPREPLY=( $(compgen -W 'mine list l query view pr pull-request id title url create update delete comment start' -- "$cur") ); fi ;;
    *) COMPREPLY=( $(compgen -W 'config team user project milestone m document docs issue completion completions' -- "$cur") ) ;;
  esac
}
complete -F _workrus workrus
