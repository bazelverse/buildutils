/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use docker_utils::{ContainerConfig, DockerUtil, WaitStrategy};
use std::process::Command;

fn get_test_container_config() -> ContainerConfig<'static> {
    ContainerConfig::new(
        "nginx",
        "nginx",
        "1.27.0",
        "0.0.0.0",
        80,
        None,
        None,
        None,
        None,
        false, // Publish ports rather than using the host network
        true,  // Keep the container running for re-use
        true,  // Keep the same container config across all env. setups.
        WaitStrategy::WaitUntilConsoleOutputContains(
            "Configuration complete; ready for start up".to_string(),
            15,
        ),
    )
}

// Always test DockerUtil with a container that is NOT used in any other integration tests.
// Otherwise, on local runs, this test suite may stop containers that are used in other tests and hence
// lead to random flaky tests and / or incorrect test results.

#[test]
fn test_docker_util() {
    let docker_util = DockerUtil::with_debug().expect("Failed to create DockerUtil");
    let container_config = get_test_container_config();
    let container_id = "nginx-80";

    test_pull(&docker_util, &container_config, container_id);

    test_start_container(&docker_util, &container_config, container_id, 80);

    test_container_exists(&docker_util, container_id);

    test_stop_container(&docker_util, container_id);

    test_container_stopped(&docker_util, container_id);
}

fn test_pull(
    docker_util: &DockerUtil,
    container_config: &ContainerConfig<'static>,
    container_id: &str,
) {
    println!("test_pull");
    let image = container_config.image();
    let tag = container_config.tag();
    let image = &format!("{image}:{tag}");
    let platform = container_config.platform();

    let res = docker_util.pull_container_image(container_id, image, platform);
    assert!(res.is_ok());
}

fn test_start_container(
    docker_util: &DockerUtil,
    container_config: &ContainerConfig<'static>,
    expected_container_id: &str,
    expected_container_port: u16,
) {
    println!("test_start_container");

    let res = docker_util.get_or_start_container(container_config);
    assert!(res.is_ok());

    let (container_id, container_port) = res.unwrap();
    assert_eq!(container_id, expected_container_id);
    assert_eq!(container_port, expected_container_port);
}

fn test_container_exists(docker_util: &DockerUtil, container_id: &str) {
    println!("test_container_exists");

    let res = docker_util.check_if_container_is_running(container_id);
    assert!(res.is_ok());
}

fn test_stop_container(docker_util: &DockerUtil, container_id: &str) {
    println!("test_stop_container");

    let delete_container = true;
    let res = docker_util.stop_container(container_id, delete_container);
    assert!(res.is_ok());

    let res = docker_util.check_if_container_is_running(container_id);

    assert!(res.is_ok());
    assert!(!res.unwrap());
}

fn test_container_stopped(docker_util: &DockerUtil, container_id: &str) {
    println!("test_container_stopped");

    let res = docker_util.check_if_container_is_running(container_id);
    assert!(res.is_ok());
    assert!(!res.unwrap());
}

// Port and container names are exclusive to the failure test below so that it cannot
// interfere with the container used in test_docker_util, which runs in parallel.
const CONFLICT_PORT: u16 = 8099;
const CONFLICT_OCCUPANT: &str = "docker-utils-occupant";
const CONFLICT_CONTENDER: &str = "docker-utils-contender";

fn get_conflict_container_config(name: &'static str) -> ContainerConfig<'static> {
    ContainerConfig::new(
        name,
        "nginx",
        "1.27.0",
        "0.0.0.0",
        CONFLICT_PORT,
        None,
        None,
        None,
        None,
        false, // Publish ports, so that the second container conflicts on the same port.
        false, // Do not reuse, so that a second start is genuinely attempted.
        false,
        WaitStrategy::NoWait,
    )
}

/// A failing `docker run` must be reported as an error.
///
/// Previously the exit status was ignored and stderr discarded, so a container that never
/// started was waited on until the wait strategy timed out, hiding the actual cause.
#[test]
fn test_start_container_reports_docker_run_failure() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");

    // Occupy the port with a first container.
    let occupant = get_conflict_container_config(CONFLICT_OCCUPANT);
    let res = docker_util.get_or_start_container(&occupant);
    assert!(res.is_ok(), "Failed to start the first container: {res:?}");

    // A second container cannot bind the same host port, so `docker run` exits non-zero.
    let contender = get_conflict_container_config(CONFLICT_CONTENDER);
    let res = docker_util.get_or_start_container(&contender);

    // Clean up both before asserting, so that a failed assertion cannot leak a container.
    // The contender is removed too: if it ever does acquire the port it would hold it for
    // good and break every later run of this test.
    remove_bare_container(&format!("{CONFLICT_CONTENDER}-{CONFLICT_PORT}"));
    remove_bare_container(&format!("{CONFLICT_OCCUPANT}-{CONFLICT_PORT}"));

    let err = res.expect_err("Starting a container on an occupied port must fail");
    let msg = err.to_string();

    // The error must name the failing command and carry Docker's own stderr, rather than
    // surfacing later as an unexplained wait timeout.
    assert!(
        msg.contains("docker run failed"),
        "Error must report the failed docker run, got: {msg}"
    );
    assert!(
        msg.contains(&CONFLICT_PORT.to_string()),
        "Error must carry Docker's stderr naming the conflicting port, got: {msg}"
    );
}

/// Starts a container directly, bypassing the crate, and returns a guard-like closure input.
///
/// No port is published, so these containers cannot collide with anything else on the host.
fn run_bare_container(name: &str) {
    let out = Command::new("docker")
        .args(["run", "--rm", "--detach", "--name", name, "nginx:1.27.0"])
        .output()
        .expect("Failed to run docker");

    assert!(
        out.status.success(),
        "Failed to start {name}: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
}

fn remove_bare_container(name: &str) {
    Command::new("docker")
        .args(["rm", "-f", name])
        .output()
        .expect("Failed to remove container");
}

/// A container ID must be matched exactly.
///
/// Docker matches the `name` filter as an unanchored regular expression, so looking up
/// `<name>-808` previously also matched `<name>-8080` and reported it as running.
#[test]
fn test_check_running_matches_the_container_id_exactly() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");

    let running = "docker-utils-exact-8080";
    let prefix_of_running = "docker-utils-exact-808";

    run_bare_container(running);

    let exact = docker_util.check_if_container_is_running(running);
    let prefix = docker_util.check_if_container_is_running(prefix_of_running);

    remove_bare_container(running);

    assert!(
        exact.expect("Failed to check the running container"),
        "The container itself must be reported as running"
    );
    assert!(
        !prefix.expect("Failed to check the prefix"),
        "{prefix_of_running} is not running and must not be matched by {running}"
    );
}

/// A container ID need not carry a `-<port>` suffix.
///
/// The port used to be parsed out of the ID with an `expect`, so checking a container whose
/// name has no numeric suffix panicked instead of returning a result.
#[test]
fn test_check_running_for_container_without_port_suffix() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "dockerutilsnoportsuffix";

    run_bare_container(name);

    let res = docker_util.check_if_container_is_running(name);

    remove_bare_container(name);

    assert!(
        res.expect("Checking a container without a port suffix must not fail"),
        "The container must be reported as running"
    );
}

/// Deleting a container that is already gone is not an error.
///
/// Containers carry `--rm`, so Docker reaps them the moment they exit. A teardown that
/// errored on the corpse would mask the failure that produced it, which is the more
/// interesting one.
#[test]
fn test_deleting_a_container_that_is_already_gone_succeeds() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");

    let res = docker_util.stop_container("docker-utils-never-started-1234", true);

    assert!(
        res.is_ok(),
        "deleting an absent container is the requested end state, got: {res:?}"
    );
}

/// Stopping without deleting still reports an absent container.
///
/// The caller named a container to stop rather than a state to reach, so silence would hide
/// a typo or a container that never started.
#[test]
fn test_stopping_a_container_that_is_not_running_is_reported() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");

    let res = docker_util.stop_container("docker-utils-never-started-1234", false);

    assert!(
        res.is_err(),
        "Stopping an absent container must be reported"
    );
}

/// A wait strategy that times out must be reported, not panicked on.
///
/// `setup_container` and `get_or_start_container` return a `Result`, so a caller has every
/// reason to expect a timeout to arrive as an `Err`. Aborting the process instead destroys
/// the evidence: it happens before the caller can collect the container's diagnostics, which
/// is the worst possible moment to panic. This test fails if either the wait or the setup
/// path goes back to `.expect()`, because a panic is a test failure.
#[test]
fn test_a_failing_wait_strategy_is_reported_rather_than_panicking() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "docker-utils-waitfail";
    let port = 6398;
    let container_id = format!("{name}-{port}");

    // Redis serves no HTTP, so this health check can only time out.
    let container_config = ContainerConfig::builder()
        .name(name)
        .image("redis")
        .tag("7-alpine")
        .url("0.0.0.0")
        .connection_port(port)
        .reuse_container(false)
        .keep_configuration(false)
        .wait_strategy(WaitStrategy::WaitForHttpHealthCheck(
            format!("http://127.0.0.1:{port}/health"),
            2,
        ))
        .build();

    let res = docker_util.setup_container(&container_config);

    remove_bare_container(&container_id);

    let err = res.expect_err("a health check that never passes must be reported");
    let msg = err.to_string();
    assert!(
        msg.contains("Timeout"),
        "the error must say the wait timed out, got: {msg}"
    );
    assert!(
        msg.contains(&container_id),
        "the error must name the container, got: {msg}"
    );
    // D3: the post-mortem rides along, so a container that died is not reported as a bare
    // timeout. Here it proves the opposite, that the container was alive and simply not
    // serving HTTP, which is equally the thing a caller needs to know.
    assert!(
        msg.contains("status=") && msg.contains("oom="),
        "the error must carry the container diagnostics, got: {msg}"
    );
}
