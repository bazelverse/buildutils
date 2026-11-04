/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use service_utils::{ServiceStartConfig, ServiceUtil, WaitStrategy};
use std::io::{Read, Write};
use std::net::TcpStream;

// Bazel already copied the binary into the tests directory during the build stage
const BAZEL_ROOT_PATH: &str = "examples/service_utils_example/tests";
// When using Cargo, the binary must be copied manually into the test folder of the crate
// cp target/debug/service_utils_example examples/service_utils_example/tests/service
const CARGO_ROOT_PATH: &str = "tests";
// Bazel renamed the binary to "service"
const PROGRAM: &str = "service";
const ADDRESS: &str = "127.0.0.1:4242";

#[test]
fn test_service() {
    // Print working directory
    println!(
        "Working directory: {}",
        std::env::current_dir().unwrap().display()
    );

    println!("Detecting build util");
    // read env variable ENV. if it exists, set the value of bazel to true, if not set to false
    let bazel_build = std::env::var("ENV").is_ok();
    if bazel_build {
        println!("Building with Bazel");
    } else {
        println!("Building with Cargo");
    }

    println!("Selecting root path");
    let root_path = get_root_path(bazel_build);
    println!("Root path: {}", root_path);

    println!(" Build service utils");
    let res = ServiceUtil::with_debug(root_path, vec![PROGRAM]);
    // dbg!(&res);
    assert!(res.is_ok());
    let util = res.unwrap();

    println!("Start service");
    let res = util.start_service_from_config(get_service_start_config());
    // dbg!(&res);
    assert!(res.is_ok(), "{res:?}");
    let pid = res.unwrap();
    println!("Service started with PID {pid}");

    println!("Test service");
    let response = get("/hello");
    println!("{response}");
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("Hello world!"), "{response}");

    // Stop what we started. Under Bazel the sandbox would do it anyway, but under Cargo there
    // is none, and a service left holding port 4242 makes the next run of this test fail.
    println!("Stop service");
    util.stop_service(pid).expect("Failed to stop the service");
}

fn get_root_path(bazel: bool) -> &'static str {
    if bazel {
        BAZEL_ROOT_PATH
    } else {
        CARGO_ROOT_PATH
    }
}

fn get_service_wait_strategy(port: u16) -> WaitStrategy {
    let url = format!("http://localhost:{port}/health");
    let timeout_in_secs = 10;
    WaitStrategy::WaitForHttpHealthCheck(url, timeout_in_secs)
}

fn get_service_start_config() -> ServiceStartConfig {
    ServiceStartConfig::builder()
        .program(PROGRAM)
        .wait_strategy(get_service_wait_strategy(4242))
        .build()
}

/// The whole response to a `GET` of `path` on the service: the example needs no HTTP client.
fn get(path: &str) -> String {
    let mut stream = TcpStream::connect(ADDRESS).expect("Failed to connect to the service");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {ADDRESS}\r\nConnection: close\r\n\r\n"
    )
    .expect("Failed to send the request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("Failed to read the response");
    response
}
