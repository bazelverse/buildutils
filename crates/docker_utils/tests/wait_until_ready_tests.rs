/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

//! Drives `WaitStrategy::WaitUntilReady` through `docker_utils` against real containers.
//!
//! Ports and names here are exclusive to this file so it cannot interfere with the other
//! integration suites running alongside it.

use docker_utils::{ContainerConfig, DockerUtil, Probe, ProbeContext, WaitStrategy};
use std::net::TcpStream;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

/// Counts probe invocations, so a test can assert the driver actually retried.
static ATTEMPTS: AtomicU32 = AtomicU32::new(0);

fn remove_container(container_id: &str) {
    Command::new("docker")
        .args(["rm", "-f", container_id])
        .output()
        .expect("Failed to remove container");
}

/// Ready once the port accepts a TCP connection.
fn port_accepts(ctx: &ProbeContext) -> Probe<(), String> {
    ATTEMPTS.fetch_add(1, Ordering::SeqCst);

    match TcpStream::connect((ctx.host(), ctx.port())) {
        Ok(_) => Probe::Ready(()),
        Err(e) => Probe::Retry(format!(
            "{}:{} not accepting yet: {e}",
            ctx.host(),
            ctx.port()
        )),
    }
}

/// Never ready, so the wait can only end in a timeout.
fn never_ready(ctx: &ProbeContext) -> Probe<(), String> {
    Probe::Retry(format!("attempt {} says no", ctx.attempt()))
}

/// Refuses outright, so the wait must stop at once rather than retry to a timeout.
fn refuses(_ctx: &ProbeContext) -> Probe<(), String> {
    Probe::Fatal("this will never work".to_string())
}

/// Builds its own tokio runtime, which is only legal with no ambient one.
///
/// This is the contract D9 exists to guarantee: a probe may do this even though
/// `setup_container` is routinely called from inside `#[test]`.
fn builds_a_runtime(ctx: &ProbeContext) -> Probe<(), String> {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => return Probe::Fatal(format!("could not build a probe runtime: {e}")),
    };

    runtime.block_on(async { port_accepts(ctx) })
}

fn config(
    name: &'static str,
    port: u16,
    strategy: WaitStrategy,
    reuse: bool,
) -> ContainerConfig<'static> {
    ContainerConfig::builder()
        .name(name)
        .image("redis")
        .tag("7-alpine")
        .url("127.0.0.1")
        .connection_port(port)
        .reuse_container(reuse)
        .keep_configuration(false)
        .wait_strategy(strategy)
        .build()
}

/// The probe gates the container: `setup_container` returns only once it reports ready.
#[test]
fn a_ready_probe_opens_the_gate() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "docker-utils-probe-ready";
    let port = 6410;
    let container_id = format!("{name}-{port}");

    ATTEMPTS.store(0, Ordering::SeqCst);

    let res = docker_util.get_or_start_container(&config(
        name,
        port,
        WaitStrategy::WaitUntilReady {
            probe: port_accepts,
            timeout_secs: 30,
            retry_delay_ms: 100,
        },
        false,
    ));

    remove_container(&container_id);

    let (id, started_port) = res.expect("a container whose probe reports ready must start");
    assert_eq!(id, container_id);
    assert_eq!(started_port, port);
    assert!(
        ATTEMPTS.load(Ordering::SeqCst) >= 1,
        "the probe must actually have been run"
    );
}

/// A probe may build its own async runtime, even from inside `#[test]`.
///
/// The driver runs the probe on a thread of its own precisely so this cannot panic with
/// "Cannot start a runtime from within a runtime".
#[test]
fn a_probe_may_build_a_runtime_inside_a_tokio_test() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "docker-utils-probe-runtime";
    let port = 6411;
    let container_id = format!("{name}-{port}");

    let res = docker_util.get_or_start_container(&config(
        name,
        port,
        WaitStrategy::WaitUntilReady {
            probe: builds_a_runtime,
            timeout_secs: 30,
            retry_delay_ms: 100,
        },
        false,
    ));

    remove_container(&container_id);

    assert!(
        res.is_ok(),
        "a probe building its own runtime must not panic: {res:?}"
    );
}

/// A probe that never reports ready ends in a timeout carrying its last message.
#[test]
fn a_probe_that_never_succeeds_times_out_with_its_own_message() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "docker-utils-probe-timeout";
    let port = 6412;
    let container_id = format!("{name}-{port}");

    let res = docker_util.get_or_start_container(&config(
        name,
        port,
        WaitStrategy::WaitUntilReady {
            probe: never_ready,
            timeout_secs: 2,
            retry_delay_ms: 100,
        },
        false,
    ));

    remove_container(&container_id);

    let msg = res
        .expect_err("a probe that never succeeds must time out")
        .to_string();
    assert!(msg.contains("Timeout"), "got: {msg}");
    assert!(
        msg.contains("says no"),
        "the probe's own message must survive: {msg}"
    );
    // The post-mortem rides along even on a probe timeout.
    assert!(
        msg.contains("status="),
        "diagnostics must be attached: {msg}"
    );
}

/// A fatal probe stops at once instead of retrying into a timeout.
#[test]
fn a_fatal_probe_is_not_retried_into_a_timeout() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "docker-utils-probe-fatal";
    let port = 6413;
    let container_id = format!("{name}-{port}");

    let res = docker_util.get_or_start_container(&config(
        name,
        port,
        // A generous timeout: if the fatal outcome were retried this would take that long
        // and report a timeout instead of the cause.
        WaitStrategy::WaitUntilReady {
            probe: refuses,
            timeout_secs: 120,
            retry_delay_ms: 100,
        },
        false,
    ));

    remove_container(&container_id);

    let msg = res
        .expect_err("a fatal probe must fail the setup")
        .to_string();
    assert!(msg.contains("this will never work"), "got: {msg}");
    assert!(
        !msg.contains("Timeout"),
        "a fatal failure must not be reported as a timeout: {msg}"
    );
}

/// A container that dies mid-wait aborts the wait rather than running out the clock.
///
/// This also exercises the reuse path: a running container is handed to the wait strategy
/// rather than returned unverified, which is what makes readiness survive a recycled runner.
///
/// This is the defect that cost two days: an OOM two seconds into a long wait reported only
/// the timeout, which is the symptom and not the cause.
#[test]
fn a_container_that_dies_mid_wait_aborts_the_wait() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "docker-utils-probe-dies";
    let port = 6414;
    let container_id = format!("{name}-{port}");

    remove_container(&container_id);

    // Already running when the setup call arrives, and exits on its own while the wait below
    // is still going.
    let out = Command::new("docker")
        .args([
            "run",
            "--detach",
            "--name",
            &container_id,
            "alpine:3.20",
            "sh",
            "-c",
            "sleep 3; exit 9",
        ])
        .output()
        .expect("Failed to run docker");
    assert!(out.status.success(), "failed to start the container");

    // reuse_container(true) takes the reuse path, which must still apply the wait strategy.
    // A long timeout: without the liveness check this would run for the full two minutes.
    let res = docker_util.get_or_start_container(&config(
        name,
        port,
        WaitStrategy::WaitUntilReady {
            probe: never_ready,
            timeout_secs: 120,
            retry_delay_ms: 100,
        },
        true,
    ));

    remove_container(&container_id);

    let msg = res
        .expect_err("a container that stopped must fail the wait")
        .to_string();
    assert!(
        msg.contains("stopped while waiting"),
        "the wait must abort on the death, not the clock: {msg}"
    );
    assert!(
        !msg.contains("Timeout"),
        "a dead container must not be reported as a timeout: {msg}"
    );
}
