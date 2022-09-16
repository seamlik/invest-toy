# Simple task runner based in PowerShell.
#
# This script is similar to a Makefile with which I can run frequently used commands. However,
# Makefile and Bash is not officially (or even possibly) available in Windows, so we should turn to
# tools that are more modern and cross-platform. Google's Ninja was my first choice, but sadly its
# rules does not allow running multiple commands.

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
    default {
        throw "Unknown task"
    }
}