# fish completion for workrus
complete -c workrus -f
complete -c workrus -n '__fish_use_subcommand' -a 'config team user project milestone m document docs issue completion completions'
complete -c workrus -n '__fish_seen_subcommand_from team' -a 'list id members create autolinks'
complete -c workrus -n '__fish_seen_subcommand_from project' -a 'list view create'
complete -c workrus -n '__fish_seen_subcommand_from milestone m' -a 'list view create update delete'
complete -c workrus -n '__fish_seen_subcommand_from document docs' -a 'list view create update delete'
complete -c workrus -n '__fish_seen_subcommand_from issue' -a 'mine list l query view pr pull-request id title url create update delete comment start'
complete -c workrus -n '__fish_seen_subcommand_from comment' -a 'list add update delete'
