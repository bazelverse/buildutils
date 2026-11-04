[//]: # (---)
[//]: # (SPDX-License-Identifier: MIT)
[//]: # (---)


# Build Utils

Utils for CI integration tests with [Bazel](https://bazel.build/) and [BuildBuddy](https://www.buildbuddy.io/), 
but can easily be used or on any other Bazel CI system. 

## Crates

### 🐳 Docker Utils

A friendly Rust crate that makes Docker container management a breeze! With docker_utils, you can:
- Start and manage Docker containers with just a few lines of code
- Configure containers flexibly with custom ports, host networking and environment variables
- Handle container lifecycle with ease (start, stop, check status)
- Get a post-mortem when a container dies, so an OOM kill is not reported as a network fault

Check out the [docker_utils documentation](crates/docker_utils/README.md) for detailed usage examples and API reference.

### 🔧 Service Utils

A lightweight Rust crate for testing service binaries efficiently. With service_utils, you can:
- Start service binaries for easy integration testing, and stop them again by PID
- Use flexible wait strategies, including your own readiness probe, to ensure services are ready
- Configure environment variables for testing different scenarios

Check out the [service_utils documentation](crates/service_utils/README.md) for detailed usage examples and API reference.

### ⏳ Wait Utils

The wait strategies both crates above are built on, usable on its own. With wait_utils, you can:
- Wait on HTTP or gRPC health checks, container logs, or a fixed duration
- Wait on your own readiness probe, for when "the port answered" is not the same as "it is ready"
- Distinguish "not yet" from "never", so a permanent failure is reported rather than retried into a timeout

Check out the [wait_utils documentation](crates/wait_utils/README.md) for detailed usage examples and API reference.

## Examples 📚

The repository includes several examples to help you get started:

### 🐳 Docker Utils Example
Demonstrates container lifecycle management using docker_utils:
- Start and manage a Postgres container
- Check container status and health
- Handle container cleanup
[View Example](examples/docker_utils_example)

### 🔧 Service Utils Example
Shows how to test service binaries using service_utils:
- Service binary management
- Wait strategy implementation
- Integration test setup
[View Example](examples/service_utils_example)


## Build commands

Cargo build work as expected for all crates. However, Bazel is configured 
as primary build system for this project. Because not everyone is familiar with Bazel, 
I made a makefile to simplify all bazel and build related tasks.

Testing under Cargo needs one step Bazel does for you. `service_utils` starts real binaries, so
its example test needs one staged on disk first; Bazel materialises it declaratively from the
test target's `data`, and Cargo has no equivalent. Use `make test_cargo`, which stages the
binaries and then runs `cargo test`. This is a plain Cargo limitation rather than a defect in
the crates.

```text
    make build          Builds the code base incrementally (fast) for dev.
    make current        Builds the current target incrementally (fast) defined in current.txt.
    make doc            Builds documentation for the project.
    make format         Formats call code according to cargo fmt style.
    make lint           Lints and formats the code of the project.
    make fix            Fixes linting issues as reported by clippy.
    make test           Tests across all crates with Bazel.
    make test_cargo     Tests across all crates with Cargo, staging binaries first.
```

For more details on the project build configuration, please read the [BUILD.md file](BUILD.md).

## Licence
This project is licensed under the [MIT license](LICENSE).

## Author
* [Marvin Hansen](https://github.com/marvin-hansen)
