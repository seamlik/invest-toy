. "$PSScriptRoot/util.ps1"

pwsh "$PSScriptRoot/format-powershell.ps1"
StopIfLastCommandFailed

cargo fmt
StopIfLastCommandFailed