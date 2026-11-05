# Attempt 2: stopped on the public-DEV manifest path

Attempt 2 used the corrected token and mask names, but the common runner still
passed `manifest.json` for public DEV rather than `dev-manifest.json`. It
stopped before writing any target reading.

The zero-byte `dev.json` and all attempt output are preserved in
[`evidence-attempt-2/`](evidence-attempt-2/). `SHA256SUMS` is
`ed23906414c6420e1162d3497ef60819f029d7cb06da07f124b14b9e4acfe833`.

