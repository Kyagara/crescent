<div align="center">
	<h1>🌙crescent</h1>
	<p>A process manager for game servers and services.</p>
	<p>
		<a href="https://crates.io/crates/crescent-cli"><img src="https://img.shields.io/crates/v/crescent-cli?style=flat-square"/></a>
		<a href="https://github.com/Kyagara/crescent/actions?query=workflow"><img src="https://img.shields.io/github/actions/workflow/status/Kyagara/crescent/ci.yaml?label=CI&style=flat-square"/></a>
		<a href="https://codecov.io/gh/Kyagara/crescent"><img src="https://img.shields.io/codecov/c/github/Kyagara/crescent?style=flat-square"/></a>
	</p>
</div>

## Wiki

Check the [wiki](https://github.com/Kyagara/crescent/wiki) for available commands and a lot more info!

## Install:

You can either get artifacts from recent [workflows](https://github.com/Kyagara/crescent/actions), binaries from [releases](https://github.com/Kyagara/crescent/releases) or install using cargo (recommended):

```bash
cargo install crescent-cli
## or
cargo install --git https://github.com/Kyagara/crescent
```

When installing crescent using cargo, default profiles will be created in crescent's main directory: `<home>/.crescent/`.

> Applications files, profiles and any important file is located in crescent's main directory.

## Todo

- More tests, 85% target.
- Reduce unwraps
- Restart command
- Async
- Attach/Log command watches the log file with the `notify` crate, it could use the application socket to receive new lines instead
- Log rotation
