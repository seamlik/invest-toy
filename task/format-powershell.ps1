function PowerShellFormat {
    param (
        $Path
    )

    $content = Get-Content -Path $Path -Raw
    Invoke-Formatter -ScriptDefinition $content | Out-File -NoNewline -FilePath $Path
}

$targets = Get-ChildItem -Path task -Recurse -Include "*.ps1"
$targets += Get-Item -Path "$PSScriptRoot/../make.ps1"
foreach ($file in $targets) {
    PowerShellFormat -Path $file
}