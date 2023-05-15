# Change Log

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

-   Added [cross](https://github.com/cross-rs/cross) configuration file for testing other architectures.
-   Added tests util module.
-   Added macOS to the ci.

### Changed

-   Profile argument in `start` now accepts a path to a file.

## [0.3.1]- 2023-05-14

### Added

-   More tests.

### Changed

-   Reworked `attach`, removed ticker.

### Fixed

-   `kill`, `stop` and `signal` now says if the subprocess wasn't found.

## [0.3.0] - 2023-05-12

### Added

-   `status` command.
-   Profiles for the `start` command, you can now make profiles and use them to start services with a preset.
-   Added some examples of profiles.
-   A `profiles` folder is created when you build crescent.
-   Added two environment variables for the subprocess, `CRESCENT_APP_NAME`, `CRESCENT_APP_ARGS` and `CRESCENT_APP_PROFILE`.
-   Added `serde` and `toml` as dependencies.

### Changed

-   Small changes to `help` on all commands.

### Fixed

-   `start` command `args` argument now actually works.
-   `send` command now accepts a `Vec<String>`, this lets you send a command with multiple strings like `cres send say hello`.
-   Subprocess is now terminated when an error creating a socket listener occurs.
-   Integration tests now use the `status` command instead of `list`, this lets most tests run in parallel without issues.
-   Process start time on `status` was showing the wrong timezone.

### Removed

-   Removed unused `temp_file` dependency.

## [0.2.0] - 2023-05-10

### Added

-   `signal` command.
-   `kill` command.
-   `stop` command.
-   Added targets for `aarch64`, `armv7` and `arm` to the CI workflow.
-   Build artifacts are available for all targets.
-   Added `libc` as a dependency.
-   Added `chrono` as a dependency.
-   More logs for the crescent process.
-   `log` now shows the corrent amount of lines printed instead of the default 200.

### Changed

-   Updated `tabled` dependency to `v1.12.0`.
-   Moved some functions around the project.
-   Renamed several modules and functions.
-   More error checking for the subprocess.
-   Removed subprocess stdin thread.
-   Refactored tests.

## [0.1.0] - 2023-05-08

Initial release on [crates.io](https://crates.io/crates/crescent-cli).
