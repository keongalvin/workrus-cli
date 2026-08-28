# PowerShell completion for workrus
Register-ArgumentCompleter -Native -CommandName workrus -ScriptBlock {
  param($wordToComplete, $commandAst, $cursorPosition)
  $words = @($commandAst.CommandElements | ForEach-Object { $_.ToString() })
  $candidates = switch ($words[1]) {
    'team' { 'list id members create autolinks' }
    'project' { 'list view create' }
    'milestone' { 'list view create update delete' }
    'm' { 'list view create update delete' }
    'document' { 'list view create update delete' }
    'docs' { 'list view create update delete' }
    'issue' { if ($words[2] -eq 'comment') { 'list add update delete' } else { 'mine list l query view pr pull-request id title url create update delete comment start' } }
    default { 'config team user project milestone m document docs issue completion completions' }
  }
  $candidates -split ' ' | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object { [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }
}
