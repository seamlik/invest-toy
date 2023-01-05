. "$PSScriptRoot/task/util.ps1"

# Tasks
switch ($args[0]) {
    format {
        Format-All
    }
    cov {
        pwsh task/cov.ps1
    }
    default {
        throw "Unknown task"
    }
}
