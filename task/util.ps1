function StopIfLastCommandFailed {
    if (!$?) {
        throw "The last command failed"
    }
}

function Format-All {
    Format-All-PowerShell

    prettier --write --ignore-path .gitignore **/*.json **/*.yaml
    StopIfLastCommandFailed

    cargo fmt
    StopIfLastCommandFailed
}

function Format-All-PowerShell {
    $powershell_files = Get-ChildItem -Path task -Recurse -Include "*.ps1"
    $powershell_files += Get-Item -Path "$PSScriptRoot/../make.ps1"
    foreach ($file in $powershell_files) {
        Format-PowerShell -Path $file
    }
}

function Format-PowerShell {
    param (
        $Path
    )

    $content = Get-Content -Path $Path -Raw
    $formatted = (Invoke-Formatter -ScriptDefinition $content).Trim()
    Out-File -InputObject $formatted -FilePath $Path
}
