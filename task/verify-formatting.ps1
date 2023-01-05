. "$PSScriptRoot/util.ps1"

Write-Output "Verifying file formatting"

function Find-Difference {
    git --no-pager diff
    StopIfLastCommandFailed
}

Format-All
$diff_output = Find-Difference
if (($null -ne $diff_output) -and ($diff_output.Trim().Length -ne 0)) {
    Find-Difference
    throw "Code not formatted. See the difference above."
}
