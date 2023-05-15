<div align="center">
	<h1>🌙crescent</h1>
    <p>Process manager for game servers or services.</p>
	<p>
		<a href="https://crates.io/crates/crescent-cli">
			<img src="https://img.shields.io/crates/v/crescent-cli?style=flat-square"/>
		</a>
		<a href="https://github.com/Kyagara/crescent/actions?query=workflow">
			<img src="https://img.shields.io/github/actions/workflow/status/Kyagara/crescent/ci.yaml?label=CI&style=flat-square"/>
		</a>
        <a href="https://codecov.io/gh/Kyagara/crescent">
			<img src="https://img.shields.io/codecov/c/github/Kyagara/crescent?style=flat-square"/>
		</a>
	</p>
	<p>
		<a href="#installation">Installation</a> •
		<a href="#commands">Commands</a> •
		<a href="https://github.com/Kyagara/crescent/blob/main/CHANGELOG.md">Changelog</a> •
		<a href="#todo">Todo</a>
	</p>
</div>

> **Warning**
> WIP!

## OS support:

-   Linux `x86_64` - Built and tested.
-   Linux `aarch64`, `armv7` and `arm` - Built.
-   macOS `x86_64` - Built.
-   Windows - Not supported.

## Installation:

You can either get artifacts from recent [workflows](https://github.com/Kyagara/crescent/actions), binaries from [releases](https://github.com/Kyagara/crescent/releases) or install using cargo:

```bash
cargo install crescent-cli
```

or

```bash
cargo install --git https://github.com/Kyagara/crescent
```

When installing crescent using cargo, default profiles will be created in crescent's main directory: `/home/<user>/.crescent/`. You can find these profiles [here](https://github.com/Kyagara/crescent/tree/main/profiles).

> Profiles and applications files are located in crescent's main directory.

## Commands:

With `start` you can launch an application by passing the file path to your executable, optionally give it a custom name with `-n` (defaults to the file name), you can pass an `interpreter` with `-i`, for example, if you have a python project you can pass `-i python3`. Arguments can be added using `-a`. Profiles can be passed with `-p <name/path>`.

> If you provide a `java` interpreter it will add a `-jar` argument automatically.

> If your arguments have spaces or `-` make sure to use quotes after `-a` like: `-a "-Xms10G -Xmx10G"`.

`list` the running applications.

`log` prints an application's `.log` file, you can specify the amount of `lines` with `-l` (defaults to 200). You can watch the file by adding `-f` flag, which will print new lines as they are added to the file.

`send` a command to the provided application.

`attach` to an application, which let's you watch logs in realtime and send commands.

`kill` (SIGKILL), `stop` (SIGTERM) or `signal <app> <sig>` to send a signal to an application.

`status` prints information about an application.

## Testing

A simple [cross](https://github.com/cross-rs/cross) configuration file is provided for testing different architectures, it simply installs `python3-minimal` before building as python is necessary to run `long_running_service`.

```bash
cross test --target aarch64-unknown-linux-gnu
```

> If you see permission errors when running tests, you can try setting the flag `CROSS_ROOTLESS_CONTAINER_ENGINE=1`.

## Why?

I wanted to learn some Rust so I decided to create a tool similar to [PM2](https://pm2.keymetrics.io/) and [mark2](https://github.com/mark2devel/mark2), these two tools saved me from a lot of headache when spinning up background services for apps and in the case of mark2, Minecraft servers.

## Should I use cres-

Not for anything in production, game servers for friends for example shouldn't be a problem. crescent does not currently support auto restarts in case of a crash or something equivalent to `pm2 save` to start apps on system startup.

## Todo

-   More tests, 85% codecov would be cool
-   Attach/Log command watches the log file with the `notify` crate, it could use the application socket to receive new lines instead
-   Lots of unwraps inside threads
-   Probably redesign the entire thing when I acquire more [knowledge](https://www.youtube.com/watch?v=jksPhQhJRoc)
