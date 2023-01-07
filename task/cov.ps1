. "$PSScriptRoot/util.ps1"

Write-Output "Running tests and generating code coverage report"

$TARGET_DIRECTORY = "target-coverage"
Remove-Item -Recurse -Path $TARGET_DIRECTORY
$COV_DIRECTORY = "$TARGET_DIRECTORY/coverage/html"
New-Item -ItemType Directory $COV_DIRECTORY

$PROFRAW_DIRECTORY = "$TARGET_DIRECTORY/profraw"
New-Item -ItemType Directory -Path $PROFRAW_DIRECTORY
$PROFRAW_DIRECTORY_ABSOLUTE = Resolve-Path -Path $PROFRAW_DIRECTORY

$Env:RUSTFLAGS = "--codegen instrument-coverage"
$Env:LLVM_PROFILE_FILE = "$PROFRAW_DIRECTORY_ABSOLUTE/default-%p-%m.profraw"
$Env:CARGO_TARGET_DIR = $TARGET_DIRECTORY
cargo test
StopIfLastCommandFailed

grcov --branch --binary-path $TARGET_DIRECTORY/debug/deps/ --output-type html --source-dir . --output-path $COV_DIRECTORY .
StopIfLastCommandFailed

Start-Process "$COV_DIRECTORY/index.html"
