# Crescent

A WIP process manager written in Rust.

## Why?

I wanted to learn some Rust so I decided to create a tool similar to [PM2](https://pm2.keymetrics.io/) and [mark2](https://github.com/mark2devel/mark2), these tools saved me from a lot of headache when spinning up background services for apps and in the case of mark2 Minecraft servers.

## What 'works' right now:

With `start` you can launch an application by passing the file path to your app, optionally give it a custom name with `-n` (defaults to the file name), you can pass an `interpreter` with `-i`, for example, if you have a python project you can pass `-i python3`. If you provide a `java` interpreter it will add a `-jar` argument automatically. Arguments can be added using `-a`, if your arguments have spaces make sure to use quotes after `-a` like `-a "-Xms10G -Xmx10G"`.

You can `list` the running applications.

You can `log` an application `.log` file, this will simply output the file in the terminal.

You can `send` a command to the provided application.

Log, PID and the application's socket are located in `/home/<user>/.crescent/apps/<app>`.

## Should I use Cres-

No. As of now, its a miracle this even does what I want. There's a LOT for me to learn to properly build what I want.

This project is constantly changing and there's a lot of things that need to be implemented and I am spending way too much time thinking about features not even written in a paper.

## Todo

-   Tests
-   TUI for an attach command
-   Profiles (add a `-p` argument to the `start` comand to pass a config file)
-   Probably redesign the entire thing when I acquire more [knowledge](https://www.youtube.com/watch?v=jksPhQhJRoc)

## License

This project is licensed under the MIT license.
