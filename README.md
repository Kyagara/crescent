<div align="center">
	<h1>🌙crescent</h1>
	<p>A wrapper for init systems to help quickly create and manage services.</p>
	<p>
		<a href="https://crates.io/crates/crescent-cli"><img src="https://img.shields.io/crates/v/crescent-cli?style=flat-square"/></a>
		<a href="https://github.com/Kyagara/crescent/actions?query=workflow"><img src="https://img.shields.io/github/actions/workflow/status/Kyagara/crescent/ci.yaml?label=CI&style=flat-square"/></a>
		<a href="https://codecov.io/gh/Kyagara/crescent"><img src="https://img.shields.io/codecov/c/github/Kyagara/crescent?style=flat-square"/></a>
	</p>
</div>

## Wiki

Check the [wiki](https://github.com/Kyagara/crescent/wiki) for available commands and a lot more info!

## Install:

When installing crescent using cargo, default profiles will be created in crescent's main directory: `$HOME/.crescent/`.

```bash
cargo install crescent-cli
## or
cargo install --git https://github.com/Kyagara/crescent
```

## Todo

Reimplementation of features already in `crescent`:

> Not sure of adding back `save` command, there are other priorities for now.

- Reimplement tests.
- Reimplement `stop` (partial) and `kill` commands.
- Rewrite `attach` command.
- Reimplement `stop_command` on `start` command and profile.

New features:

- Maybe start using subcommand logic for some commands.
- Add `edit` service and profile command.
- Add `delete` service command.
- Finish implementing `Logger`, theres no logic of selecting a logging system.
- Add methods for checking services running, many places are doing the same checks.
- Use enums on returns from `Service` and `Logger`.
- Make copying base profiles a prompt when installing/launching for the first time.
- Improve code, looks horrible.
