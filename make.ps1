. "$PSScriptRoot/task/util.ps1"

# Tasks
switch ($args[0]) {
    format {
        Format-All
    }
    run {
        node --loader ts-node/esm cli/src/main.ts
    }
    check {
        npx tsc --noEmit
        StopIfLastCommandFailed

        cargo test
        StopIfLastCommandFailed
    }
    cov {
        pwsh task/cov.ps1
    }
    default {
        throw "Unknown task"
    }
}
