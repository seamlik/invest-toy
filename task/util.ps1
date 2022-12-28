function StopIfLastCommandFailed {
    if (!$?) {
        throw "The last command failed"
    }
}
