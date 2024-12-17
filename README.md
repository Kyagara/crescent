<div align="center">
  <h1>🌙crescent</h1>
  <p>A wrapper for init systems to help quickly create and manage services.</p>
  <p><a href="https://crates.io/crates/crescent-cli"><img src="https://img.shields.io/crates/v/crescent-cli?style=flat-square"></a>
        <a href="https://github.com/Kyagara/crescent/actions?query=workflow"><img src="https://img.shields.io/github/actions/workflow/status/Kyagara/crescent/ci.yaml?label=CI&amp;style=flat-square"></a>
        <a href="https://codecov.io/gh/Kyagara/crescent"><img src="https://img.shields.io/codecov/c/github/Kyagara/crescent?style=flat-square"></a></p>
</div>

> [!WARNING] This branch is still in development and may be unstable. Only `systemd` and `journald` is supported at the moment.

# Install:

> The main directory for profiles and applications is `$HOME/.crescent/`.

```bash
cargo install crescent-cli
## or
cargo install --git https://github.com/Kyagara/crescent --branch rewrite
```

# Todo

Reimplementation of features already in the main branch:

- Tests.
- `stop` field on profiles and `start` command, a command to be sent to trigger a shutdown of the application.

New features:

- Improve service scripts, also allow customization.
- Detection method for `Service` and `Logger` at startup, return an error if the supported logging and init systems were not found.
- Add more arguments/commands to `log`, commands to manage the logs for that service for example.
- Add `delete` service/profile command.
- Though I want to keep the use of `start` for starting and/or creating new services in a single command, it might be better to move creation of services to another command.
- Maybe use more enums on returns.
