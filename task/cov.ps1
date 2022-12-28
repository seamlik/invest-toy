. "$PSScriptRoot/util.ps1"

Write-Output "Running tests and generating code coverage report"

$COV_DIRECTORY = "target/coverage/html"
New-Item -ItemType Directory $COV_DIRECTORY

$PROFRAW_DIRECTORY = "target/proraw"
New-Item -ItemType Directory -Path $PROFRAW_DIRECTORY
$PROFRAW_DIRECTORY_ABSOLUTE = Resolve-Path -Path $PROFRAW_DIRECTORY

$Env:RUSTFLAGS = "--codegen instrument-coverage"
$Env:LLVM_PROFILE_FILE = "$PROFRAW_DIRECTORY_ABSOLUTE/default-%p-%m.profraw"
cargo test
StopIfLastCommandFailed

grcov --branch --binary-path ./target/debug/deps/ --output-type html --source-dir . --output-path $COV_DIRECTORY .
StopIfLastCommandFailed

Start-Process "$COV_DIRECTORY/index.html"
