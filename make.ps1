. "$PSScriptRoot/task/util.ps1"

# Tasks
switch ($args[0]) {
    format {
        Format-All
    }
    run {
        cargo run
        StopIfLastCommandFailed
    }
    check {
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
