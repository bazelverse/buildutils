[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/service_utils

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/service_utils/latest/service_utils/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE


# Service Utils 🛠️

A simple utility for testing service binaries with ease and flexibility mainly for usage with BAZEL during CI BUILDS.

## Why?

Integration and acceptance testing in bulk quite often requires dockerized applications but as
the number of tests increase, so does the overhead from setting up and tearing down docker containers.
With Service Utils, you can easily test your services without of docker and therefore reducing test time substantially.

## Why Bazel?

While cargo is great, it reaches its limitations when a project grows large. Specifically when you need to test
a large number of services, you need to group services, start processes, set environment variables, and ensure
everything has been cleaned up after tests. While this can be done to some degree with Cargo, Bazel offers a
comprehensive solution that can be used to test a large number of services in a fast and efficient way without the need
for complex configuration.

The service util does not offer a way to copy binaries back and forth because Bazel already has a built in mechanism to
do that. The service util is stateless as to facilitate massively parallel integration testing with Bazel Remote Build
Execution (RBE).

That said, the util can be used with Cargo,
the [service util example demonstrates](../../examples/service_utils_example)
this and documents the additional steps required. See [Running under Cargo](#running-under-cargo-) below.

## Starting and stopping 🔌

`start_service` and `start_service_from_config` return the **process ID** of the service they
started. Pass it to `stop_service` when you are done:

```rust
let pid = service_util.start_service_from_config(config)?;
// ... exercise the service ...
let status = service_util.stop_service(pid)?;
assert!(status.success());
```

`stop_service` sends SIGTERM, waits for the exit and returns its status. Whatever is still running when
the `ServiceUtil` is dropped is killed and reaped, so a test leaves nothing holding its port.

A service that exits while a wait strategy runs ends the wait at once, with `ServiceExited`, instead of
timing out. To check one that must refuse to start, read its exit code:

```rust
let pid = service_util.start_service("service1", None, &WaitStrategy::NoWait, None)?;
let status = service_util.wait_for_exit(pid, Duration::from_secs(10))?;
assert_eq!(status.code(), Some(78));
```

Two options isolate a service and let a test read what it logged:

```rust
let config = ServiceStartConfig::builder()
    .program("service1")
    .wait_strategy(WaitStrategy::NoWait)
    .env_vars(vec![("NAME".into(), "value".into())])
    .clear_env(true)              // `env_vars` alone; nothing of the test's environment
    .stderr_file("/tmp/svc.log")  // created afresh
    .build();
```

Under Bazel stopping is optional housekeeping: every test runs in a sandbox that takes the
process down with it. Under Cargo there is no sandbox, so a service left running keeps holding
its port, and the next run of the same test cannot bind. That is the one difference between the
two, and `stop_service` is what closes it.

Stopping a service that has already exited is not an error, so teardown never masks the failure
that caused it. Note the process is stopped but not reaped: the crate drops the child handle on
purpose so a service can outlive the call that started it, so the PID lingers as a zombie until
your test binary exits. That is harmless, and the port is released either way.

## Running under Cargo 🦀

Bazel stages the service binary declaratively: the test target lists a `copy_file` in its
`data`, and Bazel materialises it before the test runs. Cargo has no equivalent, and a build
script cannot stand in for one because build scripts run before the workspace's other binaries
are built and have no dependency edge to them.

So the staging is done ahead of the run instead:

```bash
make test_cargo      # builds the binaries, copies them into place, then runs cargo test
```

This is a plain Cargo limitation rather than a defect in the crate: under Bazel, `make test`
needs none of it.

## Features ✨

- **Wait Strategies**: Flexible waiting mechanisms to ensure services are ready
- **Readiness Probes**: Gate on your own operation succeeding, not on a proxy for it
- **Lifecycle Control**: Start a service, get its PID back, stop it again
- **Environment Control**: Configure service environment variables
- **Error Handling**: Comprehensive error handling for service operations

## Install 🚀

Add this to your `Cargo.toml`:

```toml
[dependencies]
service_utils = "0.2"
```

### Basic Example

```rust
use service_utils::*;
 
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize ServiceUtil with the path to your binaries
    let service_util = ServiceUtil::new(
        "/path/to/binaries",
        vec!["service1", "service2"]
    )?;

    // Start a service with a wait strategy. The PID comes back so it can be stopped again.
    let pid = service_util.start_service(
        "service1",
        None,
        &WaitStrategy::WaitForDuration(5),
        None,
    )?;

    service_util.stop_service(pid)?;

    Ok(())
}
```

## Service Start Configuration

Creating a new start configuration using the builder pattern:

```rust
use service_utils::*;

    let config = ServiceStartConfig::builder()
        .program("service1")
        .wait_strategy(WaitStrategy::WaitForDuration(5))
        .env_vars(vec![("DEBUG".into(), "1".into())])
        .build();

    let pid = service_util.start_service_from_config(config)?;
```

The config also carries the address a caller would connect to, which is what a readiness probe
is handed:

```rust
    let config = ServiceStartConfig::builder()
        .program("service1")
        .host("127.0.0.1")     // defaults to "localhost"
        .port(8080)            // required only by WaitUntilReady
        .wait_strategy(WaitStrategy::WaitUntilReady {
            probe: my_service_is_ready,
            timeout_secs: 60,
            retry_delay_ms: 250,
        })
        .build();
```

The address lives in the config because it is a chicken and egg problem: the caller cannot
connect until it knows the port, and the driver cannot start the service until the caller has
chosen one. The config is what breaks it, by being the single place both sides read.

## Bazel Test configuration

A Rust test configuration for testing with the service util and Bazel requires three segments:
1) Imports i.e. the Bazel rules to load 
2) Test suite
3) Copy the binary to the target directory

For example, the configuration below is taken from the [service_utils_example](../../examples/service_utils_example):

```python
# 1) Imports   
load("@aspect_bazel_lib//lib:copy_file.bzl", "copy_file")
load("@rules_rust//rust:defs.bzl", "rust_test_suite")

# 2) Test suite 
rust_test_suite(
    name = "tests",
    srcs = glob([
        "*_tests.rs",
    ]),
    data = [
        ":copy_service",  # Copies the service binary into the test folder
    ],
    tags = [ # Tags are used to filter and select tests to run 
        "integration-test",
        "service_utils_example",
    ],
    visibility = ["//visibility:public"],
    deps = [
        # Crate under test
        "//alias:service_example",
        # Internal crates
        "//alias:service_utils",
        # External crates
        "//thirdparty/crates:reqwest",
        "//thirdparty/crates:serde",
    ],
)

# 3) Copy the binary   
copy_file(
    name = "copy_service",  # label to this rule. Used in the data attribute
    src = "//alias:service_example",  # Alias is defined in file: alias/BUILD.bazel
    out = "service",  # Name of the output binary
    is_executable = True,  # Must always set to true otherwise the service cannot be started.
)
```  

## Wait Strategies 🕒

The crate provides several wait strategies through the `wait_utils` dependency:

- `NoWait`: Return as soon as the service is started.
- `WaitForDuration(u64)`: Wait for a specified number of seconds
- `WaitForHttpHealthCheck(String, u64)`: Wait until an HTTP request to the given URL or until a timeout occurs.
- `WaitForGrpcHealthCheck(String, u64)`, with the `grpc` feature: Wait until a gRPC health request to the given URL or until a timeout occurs.
- `WaitUntilReady { probe, timeout_secs, retry_delay_ms }`: Wait until your own probe reports ready.

`WaitUntilConsoleOutputContains` is not honoured here: it reads container logs, and this crate
starts local binaries rather than containers. It is reported as an unsupported strategy rather
than silently waiting.

### Readiness probes

Every other strategy is a fixed predicate this crate implements. None of them can express "the
operation I am about to perform succeeds", which is the only predicate that is never wrong: a
service commonly accepts connections seconds before it accepts real work. A probe closes that
gap:

```rust
use service_utils::{Probe, ProbeContext};

fn my_service_is_ready(ctx: &ProbeContext) -> Probe<(), String> {
    match TcpStream::connect((ctx.host(), ctx.port())) {
        Ok(_) => Probe::Ready(()),
        Err(e) => Probe::Retry(format!("attempt {}: {e}", ctx.attempt())),
    }
}
```

The outcome is three-way on purpose. `Probe::Fatal` stops the wait at once, so a permanent
failure is reported as itself rather than retried into a timeout that describes the symptom
instead of the cause.

A probe is a plain `fn` pointer, so it cannot capture state; the address it needs comes from
the `ProbeContext`. It is synchronous by contract and runs on a thread of its own, so it may
build its own async runtime whatever your test runs on.

## Error Handling 🚨

The crate uses a dedicated `ServiceUtilError` type that covers various failure scenarios:

- Binary not found
- Service start failure
- Service stop failure
- Service exited before it was ready, naming how
- Wait strategy timeout, carrying the probe's own last message
- Unsupported wait strategy for this driver
- Environment configuration errors

## Debug Mode 🔍

Enable debug mode for additional logging and information:

```rust
let service_util = ServiceUtil::with_debug(
    "/path/to/binaries",
    vec!["service1"]
)?;
```

## Related Crates 📦

- `wait_utils`: Provides wait strategies used by this crate
- `docker_utils`: Docker container management utilities

## Contributing

Contributions are welcome! Feel free to:

- Report issues
- Submit pull requests
- Suggest new features
- Improve documentation

## Licence

This project is licensed under the [MIT license](LICENSE).

## Author

* [Marvin Hansen](https://github.com/marvin-hansen)
* Contact: https://deepcausality.com/contact/
* Github GPG key ID: 369D5A0B210D39BC
