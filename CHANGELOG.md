# Change Log

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- `complete <terminal>` command to create a basic completions file for the selected terminal. `cres complete bash > /usr/share/bash-completion/completions/cres`.

- `strip-ansi-escapes` as a dependency.

### Changed

- Log lines from the `attach` command with ANSI code are now escaped using `strip-ansi-escapes`.

## [0.5.0] - 2023-06-23

### Added

- `profile` command, prints the provided profile, accepts a `--json`/`-j` to print the profile as prettified json.
- `save` command, saves all currently running applications to a `apps.json` file.
- `--saved` flag to `start`, will try to start all applications saved in the `apps.json` file.
- More helper functions like `get_socket_dir` and `check_app_exists`.

### Changed

- Changed the way we verify if a process is from crescent, instead of env vars, all checks are made through the application socket.
- `stop`, `status` now uses a custom socket event.
- `stop` can now send a custom stop command if provided in a profile.
- `stop` now has a `--force`/`-f` flag to bypass a user defined stop command and send a SIGTERM signal.
- Moved `test_utils` to a separate crate.
- Refactored some tests.
- Improved a bit error handling on `build.rs`.

### Removed

- Enviroment variables.

## [0.4.2] - 2023-06-17

### Changed

- Changed the project license from `MIT` to `Apache-2.0`.
- 'Uptime' in `status` and `list` now display seconds/minutes/hours/days.

### Fixed

- CI typos.

## [0.4.1] - 2023-06-12

### Added

- Added `--flush` flag to `log` command, which will clear the log file contents.
- Added a check to see if the executable path was provided.
- Added build for Intel and Arm macOs to ci.
- Added [Velocity Proxy](https://github.com/PaperMC/Velocity) profile.

### Fixed

- Minecraft profiles missing `-jar` at the end of the interpreter arguments.

## [0.4.0] - 2023-05-17

### Added

- Added [cross](https://github.com/cross-rs/cross) configuration file for testing other architectures.
- Added tests util module.
- Added macOS `x86_64` and `aarch64` to the ci.
- Added `serde_json` as a dependency.
- More tests.
- Command history for `attach`, pressing up or down will scroll through past commands for that subprocess.
- Added macos `x86` tests to ci.
- Profile fields will be overwritten if you a flag already set by the profile.
- Added environment variable `CRESCENT_APP_INTERPRETER_ARGS`.

### Changed

- Profiles file format changed to json.
- Profile argument in `start` now accepts a path to a file.
- The subprocess socket now uses serialized structs using `serde_json` for communication.
- Changed compressed ci artifacts format from `.tar.gz` to `.zip`.
- All default profiles changed to match new `start` arguments.

### Removed

- Removed `toml` dependency.

## [0.3.1] - 2023-05-14

### Added

- More tests.

### Changed

- Reworked `attach`, removed ticker.

### Fixed

- `kill`, `stop` and `signal` now says if the subprocess wasn't found.

## [0.3.0] - 2023-05-12

### Added

- `status` command.
- Profiles for the `start` command, you can now make profiles and use them to start services with a preset.
- Added some examples of profiles.
- A `profiles` folder is created when you build crescent.
- Added two environment variables for the subprocess, `CRESCENT_APP_NAME`, `CRESCENT_APP_ARGS` and `CRESCENT_APP_PROFILE`.
- Added `serde` and `toml` as dependencies.

### Changed

- Small changes to `help` on all commands.

### Fixed

- `start` command `args` argument now actually works.
- `send` command now accepts a `Vec<String>`, this lets you send a command with multiple strings like `cres send say hello`.
- Subprocess is now terminated when an error creating a socket listener occurs.
- Integration tests now use the `status` command instead of `list`, this lets most tests run in parallel without issues.
- Process start time on `status` was showing the wrong timezone.

### Removed

- Removed unused `temp_file` dependency.

## [0.2.0] - 2023-05-10

### Added

- `signal` command.
- `kill` command.
- `stop` command.
- Added targets for `aarch64`, `armv7` and `arm` to the CI workflow.
- Build artifacts are available for all targets.
- Added `libc` as a dependency.
- Added `chrono` as a dependency.
- More logs for the crescent process.
- `log` now shows the corrent amount of lines printed instead of the default 200.

### Changed

- Updated `tabled` dependency to `v1.12.0`.
- Moved some functions around the project.
- Renamed several modules and functions.
- More error checking for the subprocess.
- Removed subprocess stdin thread.
- Refactored tests.

## [0.1.0] - 2023-05-08

Initial release on [crates.io](https://crates.io/crates/crescent-cli).
