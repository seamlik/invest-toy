. "$PSScriptRoot/task/util.ps1"

function Format-PowerShell {
    param (
        $Path
    )

    $content = Get-Content -Path $Path -Raw
    Invoke-Formatter -ScriptDefinition $content | Out-File -NoNewline -FilePath $Path
}

function StopIfLastCommandFailed {
    if (!$?) {
        throw "The last command failed"
    }
}

# Tasks
switch ($args[0]) {
    format {
        Format-PowerShell -Path make.ps1
        eslint --fix cli/
        prettier --write **/*.json **/*.yaml
    }
    run {
        node --loader ts-node/esm cli/src/main.ts
    }
    check {
        npx tsc --noEmit
        StopIfLastCommandFailed

        npx jest
        StopIfLastCommandFailed
    }
    cov {
        pwsh task/cov.ps1
    }
    default {
        throw "Unknown task"
    }
}