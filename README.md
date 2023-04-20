# Crescent

A WIP process manager written in Rust.

## Why?

I wish to learn Rust so I decided to create a tool similar to [PM2](https://pm2.keymetrics.io/) and [mark2](https://github.com/mark2devel/mark2/blob/master/INSTALL.md) that very much saved me from a lot of headache when spinning up services.

## What 'works' right now:

You can `start` an application by passing the file path and optionally flags such as `-n` for a custom application name (defaults to the file name) and `-c` for a `command`, running a python application? Add a `-c` `python`, `python2`, `python3` and so on. If you provide a java command it will add a `-jar` argument. Arguments can be added using `-a`, if your arguments have spaces make sure to use quotes after `-a` like `-a "-Xms10G -Xmx10G"`.

You can `list` the running applications.

You can `log` an application `.log` file, this will simply output the file in the terminal.

You can `send` a command to the provided application.

Log, PID and the application's socket are located in `/home/<user>/.crescent/<app>`.

## Should I use Cres-

No. As of now, its a miracle this even does what I want. There's a LOT for me to learn to properly build what I want.

This project is constantly changing and there's a lot of things that need to be implemented and I am spending way too much time thinking about features not even written in a paper.

## Todo

-   Error checking, literally any error checking whatsoever
-   Tests
-   Github Actions for testing, linting and publishing
-   TUI for an attach command
-   Profiles (add a `-p` argument to the `start` comand to pass a config file)
-   Probably redesign the entire thing when I acquire [knowledge](https://www.youtube.com/watch?v=jksPhQhJRoc)

## License

This project is licensed under the MIT license.
