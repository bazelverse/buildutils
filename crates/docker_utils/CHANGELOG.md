# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/bazelverse/buildutils/compare/docker_utils-v0.4.0...docker_utils-v0.4.1) - 2026-11-04

### Other

- updated README.md
- release v0.4.0

## [0.4.0](https://github.com/bazelverse/buildutils/compare/docker_utils-v0.3.5...docker_utils-v0.4.0) - 2026-11-04

### Added

- [**breaking**] std-only wait, service and docker utils; migrate to rules_rs

## [0.3.5](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.3.4...docker_utils-v0.3.5) - 2026-08-21

### Other

- *(docker_utils)* release v0.3.4

## [0.3.4](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.3.3...docker_utils-v0.3.4) - 2026-08-20

### Added

- *(docker_utils)* mount volumes into a container

## [0.3.3](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.3.2...docker_utils-v0.3.3) - 2026-07-29

### Added

- honour a caller-supplied readiness probe in both drivers

### Other

- Merge remote-tracking branch 'origin/main'

## [0.3.2](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.3.1...docker_utils-v0.3.2) - 2026-07-29

### Added

- add container diagnostics and a caller-supplied readiness probe

## [0.3.1](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.3.0...docker_utils-v0.3.1) - 2026-07-28

### Fixed

- *(wait_utils)* respect the HTTP status and stderr when waiting

## [0.3.0](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.2.4...docker_utils-v0.3.0) - 2026-07-28

### Added

- *(docker_utils)* [**breaking**] add host networking and surface Docker failures

### Other

- Merge remote-tracking branch 'origin/main'
- Fixed more lints and clippies
- Fixed error reporting in DockerUitls
- Code formatting, fixes, and linting

## [0.2.4](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.2.3...docker_utils-v0.2.4) - 2025-10-07

### Other

- update Cargo.toml dependencies

## [0.2.3](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.2.2...docker_utils-v0.2.3) - 2025-01-20

### Other

- update Cargo.toml dependencies

## [0.2.2](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.2.1...docker_utils-v0.2.2) - 2025-01-20

### Other

- Excluded all Bazel files from crate release to prevent issues when vendoring these crates with Bazel.

## [0.2.1](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.2.0...docker_utils-v0.2.1) - 2025-01-19

### Other

- Updated README.md across all crates
- Merge remote-tracking branch 'origin/main'
- Added project wide copyright

## [0.2.0](https://github.com/marvin-hansen/buildutils/compare/docker_utils-v0.1.0...docker_utils-v0.2.0) - 2025-01-19

### Other

- Updated README.md
- Updated tests and documentation
