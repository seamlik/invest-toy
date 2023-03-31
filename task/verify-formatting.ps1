. "$PSScriptRoot/util.ps1"

function Find-Difference {
    git --no-pager diff
    StopIfLastCommandFailed
}

pwsh "$PSScriptRoot/format-all.ps1"
$diff_output = Find-Difference
if (($null -ne $diff_output) -and ($diff_output.Trim().Length -ne 0)) {
    throw "Code not formatted"
}
