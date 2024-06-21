<div align="center">
	<h1>🌙crescent</h1>
	<p>A wrapper for init systems to help quickly create and manage services.</p>
	<p>
		<a href="https://crates.io/crates/crescent-cli"><img src="https://img.shields.io/crates/v/crescent-cli?style=flat-square"/></a>
		<a href="https://github.com/Kyagara/crescent/actions?query=workflow"><img src="https://img.shields.io/github/actions/workflow/status/Kyagara/crescent/ci.yaml?label=CI&style=flat-square"/></a>
		<a href="https://codecov.io/gh/Kyagara/crescent"><img src="https://img.shields.io/codecov/c/github/Kyagara/crescent?style=flat-square"/></a>
	</p>
</div>

> [!WARNING]
> This branch is still in development and may be unstable. Only `systemd` and `journald` is supported at the moment.

## Install:

> The main directory for profiles and applications is `$HOME/.crescent/`.

```bash
cargo install crescent-cli
## or
cargo install --git https://github.com/Kyagara/crescent
```

## Todo

Reimplementation of features already in `crescent`:

- Reimplement tests.
- `stop` field on profiles and `start` command, a command to be sent when triggering a shutdown.

New features:

- Custom implementation of a logger widget.
- Turn the python test program to a rust binary.
- Improve service scripts, also allow for more customization.
- Might look for another TUI library.
- Detection method for `init` systems and logging systems.
- Add `delete` service/profile command.
- Finish implementing `Logger`, theres no logic of selecting a logging system and setting it for a service.
- Add more arguments/commands to `log`.
- Maybe use more enums on returns.
- Decrease amount of crates.
- Improve code, looks horrible.
