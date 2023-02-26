# rdo

[![Rust CI](https://github.com/nseguin42/rdo/actions/workflows/rust-ci.yaml/badge.svg)](https://github.com/nseguin42/rdo/actions/workflows/rust-ci.yaml)
![Stars](https://img.shields.io/github/stars/nseguin42/rdo)
[![License](https://img.shields.io/github/license/nseguin42/rdo)](https://github.com/nseguin42/rdo/blob/main/LICENSE)

rdo is a tool for running commands with dependencies.

## Example Usage

### Config: `config/config.toml`

```toml
[log]
level = "info"

[script.test_1]
path = "scripts/test.sh"
args = ["test_1", "Run after all other tests."]
type = "Bash"
dependencies = ["test_2", "test_3", "test_4", "test_5", "test_6"]

[script.test_2]
path = "scripts/test.sh"
args = ["test_2", "Run after test_6."]
type = "Bash"
dependencies = ["test_6"]

[script.test_3]
path = "scripts/test.sh"
args = ["test_3", "Run after test_2."]
type = "Bash"
dependencies = ["test_2"]

[script.test_4]
path = "scripts/test.sh"
args = ["test_4", "Run after test_5 and test_6."]
type = "Bash"
dependencies = ["test_5", "test_6"]

[script.test_5]
path = "scripts/test.sh"
args = ["test_5", "Run after test_6."]
type = "Bash"
dependencies = ["test_6"]

[script.test_6]
path = "scripts/test.sh"
args = ["test_6", "Run anytime."]
type = "Bash"
```

### Script: `scripts/test.sh`

```bash
#!/bin/bash
echo "$1 | $2"
```

### Output: `cargo run`

```
INFO rdo::logger > Logger initialized
INFO rdo::script > stdout: test_6 | Run anytime.
INFO rdo::script > stdout: test_5 | Run after test_6.
INFO rdo::script > stdout: test_4 | Run after test_5 and test_6.
INFO rdo::script > stdout: test_2 | Run after test_6.
INFO rdo::script > stdout: test_3 | Run after test_2.
INFO rdo::script > stdout: test_1 | Run after all other tests.
```

## TODO

- [ ] Improve console output / human-friendliness
- [ ] Improve error handling
- [ ] Add support for more script types
- [ ] Add script templates / connectors
    - Package update script
    - With retries
    - With timeout
- [x] Add interactive console with streaming output
- [ ] Add support for running scripts in parallel
- [ ] Add more complex dependency logic
    - `IF` / `ELSE` / `AND` / `OR` / `NOT` syntax
    - "Necessary" and "sufficient" syntax
- [ ] Post execution summary
- [x] Interruptible execution
