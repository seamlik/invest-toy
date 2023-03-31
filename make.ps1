. "$PSScriptRoot/task/util.ps1"

# Tasks
switch ($args[0]) {
    format {
        pwsh "$PSScriptRoot/task/format-all.ps1"
    }
    cov {
        pwsh "$PSScriptRoot/task/cov.ps1"
    }
    default {
        throw "Unknown task"
    }
}
