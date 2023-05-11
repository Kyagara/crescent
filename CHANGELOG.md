# Change Log

All notable changes to this project will be documented in this file.

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
