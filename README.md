<div align="center">
  <h1>🌙crescent</h1>
  <p>A wrapper for init systems to help quickly create and manage services.</p>
  <p><a href="https://crates.io/crates/crescent-cli"><img src="https://img.shields.io/crates/v/crescent-cli?style=flat-square"></a>
        <a href="https://github.com/Kyagara/crescent/actions?query=workflow"><img src="https://img.shields.io/github/actions/workflow/status/Kyagara/crescent/ci.yaml?label=CI&amp;style=flat-square"></a>
        <a href="https://codecov.io/gh/Kyagara/crescent"><img src="https://img.shields.io/codecov/c/github/Kyagara/crescent?style=flat-square"></a></p>
</div>

> [!WARNING]
> This project is still in development and may be unstable. Only `systemd` and `journald` is supported.

# Install:

> The main directory for profiles and services is `$HOME/.crescent/`.

```bash
cargo install crescent-cli
## or
cargo install --git https://github.com/Kyagara/crescent
```

# Todo

Reimplementation of old features:

- Tests.
- `stop` field on profiles and `start` command, a command to be sent to trigger a shutdown of the application.

Planned:

- Flag in main to always confirm prompts.
- Maybe add environment variable for the main crescent directory, retrieving the user's home directory while in root returns "/root".
- Fix some commands not erroring when failing to send commands to a system service.
- Save information about the service in a file inside the service folder.
- Add more arguments/commands to `log`, commands to manage the logs for that service for example.
- Add `delete` service/profile command.
- Maybe use more enums on returns.
