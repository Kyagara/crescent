<div align="center">
	<h1>🌙crescent</h1>
    <p>Process manager for game servers or services.</p>
	<p>
		<a href="https://github.com/Kyagara/crescent/actions?query=workflow">
			<img src="https://img.shields.io/github/actions/workflow/status/Kyagara/crescent/ci.yaml?label=CI&style=flat-square"/>
		</a>
        <a href="https://codecov.io/gh/Kyagara/crescent">
			<img src="https://img.shields.io/codecov/c/github/Kyagara/crescent?style=flat-square"/>
		</a>
	</p>
</div>

> **Warning**
> WIP!

## Installation:

```bash
cargo install --git https://github.com/Kyagara/crescent
```

## OS support:

My main focus is Linux, crescent is built and tested on ubuntu amd64 and should work on aarch64. MacOs is not tested. Windows is not supported.

## What works right now:

With `start` you can launch an application by passing the file path to your executable, optionally give it a custom name with `-n` (defaults to the file name), you can pass an `interpreter` with `-i`, for example, if you have a python project you can pass `-i python3`. Arguments can be added using `-a`.

> If you provide a `java` interpreter it will add a `-jar` argument automatically.

> If your arguments have spaces make sure to use quotes after `-a` like: `-a "-Xms10G -Xmx10G"`.

You can `list` the running applications.

You can `log` an application's `.log` file, you can specify the amount of `lines` with `-l` (defaults to 200). After printing the log, it will watch the file for any new lines added to it.

You can `send` a command to the provided application.

You can `attach` to an application, which let's you watch logs in realtime and send commands.

Log, PID and the application's socket are located in `/home/<user>/.crescent/apps/<app>`.

## Why?

I wanted to learn some Rust so I decided to create a tool similar to [PM2](https://pm2.keymetrics.io/) and [mark2](https://github.com/mark2devel/mark2), these two tools saved me from a lot of headache when spinning up background services for apps and in the case of mark2, Minecraft servers.

## Should I use cres-

Not for anything in production, game servers for friends for example shouldn't be a problem. crescent does not currently support auto restarts in case of a crash or something equivalent to `pm2 save` to start apps on system startup.

## Todo

-   More tests
-   Attach command watches the log file with the `notify` crate, it could use the application socket to receive new lines instead
-   Lots of unwraps inside threads
-   Profiles (add a `-p` argument to the `start` command to pass a config file)
-   There should be more logs for the daemonized crescent process, maybe a separate log/socket for it
-   Probably redesign the entire thing when I acquire more [knowledge](https://www.youtube.com/watch?v=jksPhQhJRoc)

## License

This project is licensed under the MIT license.
