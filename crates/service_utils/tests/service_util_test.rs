/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

//! Drives `ServiceUtil` against a real process.
//!
//! The service binary is a shell script this file writes at run time, rather than a compiled
//! artefact copied into place. Bazel stages such artefacts automatically and Cargo does not,
//! so a test depending on one passes under Bazel and fails under Cargo. Creating it here
//! keeps the suite identical under both.

use service_utils::{
    Probe, ProbeContext, ServiceStartConfig, ServiceUtil, ServiceUtilError, WaitStrategy,
};
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// Must be a `&'static str` because that is what `ServiceUtil` takes.
const ROOT_PATH: &str = "/tmp/service_utils_test_bin";
const PROGRAM: &str = "dummy_service";
/// Scripts of their own, so tests running side by side never rewrite one another's binary.
const SCRIPT_ROOT: &str = "/tmp/service_utils_test_scripts";

static ATTEMPTS: AtomicU32 = AtomicU32::new(0);

/// Writes an executable script that stays alive, and returns once it exists.
fn stage_service_binary() {
    fs::create_dir_all(ROOT_PATH).expect("Failed to create the test bin directory");

    let path = format!("{ROOT_PATH}/{PROGRAM}");
    let mut file = fs::File::create(&path).expect("Failed to create the test binary");
    file.write_all(b"#!/bin/sh\nsleep 30\n")
        .expect("Failed to write the test binary");
    drop(file);

    let mut perms = fs::metadata(&path)
        .expect("Failed to stat the test binary")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("Failed to chmod the test binary");
}

/// Ready as soon as it is called, recording that it ran at all.
fn always_ready(_ctx: &ProbeContext) -> Probe<(), String> {
    ATTEMPTS.fetch_add(1, Ordering::SeqCst);
    Probe::Ready(())
}

/// Records the address it was handed, so a test can prove the config reached the probe.
static SEEN_PORT: AtomicU32 = AtomicU32::new(0);

fn records_the_address(ctx: &ProbeContext) -> Probe<(), String> {
    SEEN_PORT.store(u32::from(ctx.port()), Ordering::SeqCst);
    Probe::Ready(())
}

/// Never ready, so a wait can only end in a timeout.
fn never_ready(_ctx: &ProbeContext) -> Probe<(), String> {
    Probe::Retry("not yet".to_string())
}

/// Returns a port with nothing listening on it.
fn a_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    listener.local_addr().expect("Failed to read addr").port()
}

#[test]
fn a_missing_binary_is_reported() {
    stage_service_binary();

    let res = ServiceUtil::new(ROOT_PATH, vec!["no_such_program"]);

    let err = res.expect_err("a binary that is not there must be reported");
    assert!(
        err.to_string().contains("Binary not found"),
        "Unexpected error: {err}"
    );
}

#[test]
fn a_present_binary_is_accepted() {
    stage_service_binary();

    let res = ServiceUtil::new(ROOT_PATH, vec![PROGRAM]);

    assert!(res.is_ok(), "a staged binary must be accepted: {res:?}");
}

/// The readiness probe is honoured here, not only in docker_utils.
///
/// A strategy only one driver can run is the defect this crate is trying not to repeat.
#[test]
fn a_readiness_probe_is_honoured() {
    stage_service_binary();
    ATTEMPTS.store(0, Ordering::SeqCst);

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program(PROGRAM)
        .port(a_free_port())
        .wait_strategy(WaitStrategy::WaitUntilReady {
            probe: always_ready,
            timeout_secs: 10,
            retry_delay_ms: 10,
        })
        .build();

    let res = service_util.start_service_from_config(config);

    assert!(res.is_ok(), "a ready probe must start the service: {res:?}");
    assert!(
        ATTEMPTS.load(Ordering::SeqCst) >= 1,
        "the probe must actually have been run"
    );
}

/// The address configured by the caller is the address the probe receives.
///
/// This is the whole point of putting it in the config: the caller cannot connect until it
/// knows the port and the driver cannot bind until the caller has chosen one, so the config
/// is what both sides read.
#[test]
fn the_configured_address_reaches_the_probe() {
    stage_service_binary();
    let port = a_free_port();
    SEEN_PORT.store(0, Ordering::SeqCst);

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program(PROGRAM)
        .host("127.0.0.1")
        .port(port)
        .wait_strategy(WaitStrategy::WaitUntilReady {
            probe: records_the_address,
            timeout_secs: 10,
            retry_delay_ms: 10,
        })
        .build();

    service_util
        .start_service_from_config(config)
        .expect("the service must start");

    assert_eq!(
        SEEN_PORT.load(Ordering::SeqCst),
        u32::from(port),
        "the probe must be handed the port the config declared"
    );
}

/// Without a port there is no address, and saying so beats probing a made-up one.
#[test]
fn a_readiness_probe_without_a_port_is_reported() {
    stage_service_binary();

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program(PROGRAM)
        .wait_strategy(WaitStrategy::WaitUntilReady {
            probe: always_ready,
            timeout_secs: 10,
            retry_delay_ms: 10,
        })
        .build();

    let err = service_util
        .start_service_from_config(config)
        .expect_err("a probe with no address must be reported");

    assert!(
        err.to_string().contains("no port is set"),
        "the error must say what is missing: {err}"
    );
}

/// A probe that never succeeds ends in a timeout rather than hanging.
#[test]
fn a_readiness_probe_that_never_succeeds_times_out() {
    stage_service_binary();

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program(PROGRAM)
        .port(a_free_port())
        .wait_strategy(WaitStrategy::WaitUntilReady {
            probe: never_ready,
            timeout_secs: 1,
            retry_delay_ms: 10,
        })
        .build();

    let err = service_util
        .start_service_from_config(config)
        .expect_err("a probe that never succeeds must time out");

    assert!(err.to_string().contains("Timeout"), "got: {err}");
    assert!(
        err.to_string().contains("not yet"),
        "the probe's own message must survive: {err}"
    );
}

/// A container log strategy has no container here, and is reported rather than waited on.
#[test]
fn a_console_output_strategy_is_reported_as_unsupported() {
    stage_service_binary();

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program(PROGRAM)
        .wait_strategy(WaitStrategy::WaitUntilConsoleOutputContains(
            "ready".to_string(),
            1,
        ))
        .build();

    let err = service_util
        .start_service_from_config(config)
        .expect_err("service_utils cannot read container logs");

    assert!(
        err.to_string().contains("Unsupported wait strategy"),
        "Unexpected error: {err}"
    );
}

/// Starting a program that was never declared is reported.
#[test]
fn starting_an_undeclared_program_is_reported() {
    stage_service_binary();

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program("some_other_program")
        .wait_strategy(WaitStrategy::NoWait)
        .build();

    let err = service_util
        .start_service_from_config(config)
        .expect_err("an undeclared program must be reported");

    assert!(
        err.to_string().contains("Binary has not been added"),
        "Unexpected error: {err}"
    );
}

/// A started service can be stopped again by the PID it reported.
///
/// A stopped service is gone by the time `stop_service` returns, reaped, with its port released.
#[test]
fn a_started_service_can_be_stopped_by_its_pid() {
    stage_service_binary();

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program(PROGRAM)
        .wait_strategy(WaitStrategy::NoWait)
        .build();

    let pid = service_util
        .start_service_from_config(config)
        .expect("the service must start");

    assert!(pid > 0, "a started service must report its PID");
    assert!(
        process_is_alive(pid),
        "the service must be running before it is stopped"
    );

    service_util
        .stop_service(pid)
        .expect("the service must stop");

    assert!(
        !process_is_running(pid),
        "the service must no longer be running after stop_service"
    );
}

/// Stopping something already gone is the requested end state, not an error.
#[test]
fn stopping_an_already_stopped_service_succeeds() {
    stage_service_binary();

    let service_util =
        ServiceUtil::new(ROOT_PATH, vec![PROGRAM]).expect("Failed to create ServiceUtil");

    let config = ServiceStartConfig::builder()
        .program(PROGRAM)
        .wait_strategy(WaitStrategy::NoWait)
        .build();

    let pid = service_util
        .start_service_from_config(config)
        .expect("the service must start");

    service_util
        .stop_service(pid)
        .expect("the first stop works");

    let res = service_util.stop_service(pid);

    assert!(
        res.is_ok(),
        "stopping an already stopped service must succeed: {res:?}"
    );
}

/// Writes an executable script running `body`, under a name of its own, and returns the name.
fn stage_script(name: &'static str, body: &str) -> &'static str {
    fs::create_dir_all(SCRIPT_ROOT).expect("Failed to create the script directory");
    let path = format!("{SCRIPT_ROOT}/{name}");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("Failed to write the script");
    let mut perms = fs::metadata(&path).expect("Failed to stat").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("Failed to chmod the script");
    name
}

fn scripts(name: &'static str) -> ServiceUtil {
    ServiceUtil::new(SCRIPT_ROOT, vec![name]).expect("Failed to create ServiceUtil")
}

/// A service that refuses to start tells how: the exit code a caller asserts on.
#[test]
fn an_exit_code_can_be_read() {
    let name = stage_script("exits_78", "exit 78");
    let util = scripts(name);
    let pid = util
        .start_service(name, None, &WaitStrategy::NoWait, None)
        .expect("the service must start");
    let status = util
        .wait_for_exit(pid, Duration::from_secs(10))
        .expect("the service must exit");
    assert_eq!(status.code(), Some(78));
    let again = util
        .wait_for_exit(pid, Duration::ZERO)
        .expect("the status stays");
    assert_eq!(again.code(), Some(78), "the status can be read again");
}

/// A service still running when the wait ends is reported, not waited on forever.
#[test]
fn waiting_for_a_running_service_to_exit_times_out() {
    let name = stage_script("keeps_running", "sleep 30");
    let util = scripts(name);
    let pid = util
        .start_service(name, None, &WaitStrategy::NoWait, None)
        .expect("the service must start");
    let res = util.wait_for_exit(pid, Duration::from_millis(50));
    assert!(
        matches!(res, Err(ServiceUtilError::ServiceNotRunning(_))),
        "{res:?}"
    );
}

/// A PID this `ServiceUtil` did not start is refused, so it signals nobody else's process.
#[test]
fn a_pid_it_did_not_start_is_refused() {
    let name = stage_script("never_started", "exit 0");
    let util = scripts(name);
    let unknown = std::process::id();
    assert!(matches!(
        util.stop_service(unknown),
        Err(ServiceUtilError::ServiceNotRunning(_))
    ));
    assert!(util.wait_for_exit(unknown, Duration::ZERO).is_err());
}

/// SIGTERM reaches the service, so a test can check its shutdown's own exit code.
#[test]
fn stopping_a_service_returns_how_it_exited() {
    let name = stage_script(
        "traps_term",
        "trap 'exit 7' TERM\nwhile :; do sleep 0.05; done",
    );
    let util = scripts(name);
    let pid = util
        .start_service(name, None, &WaitStrategy::WaitForDuration(1), None)
        .expect("the service must start");
    let status = util.stop_service(pid).expect("the service must stop");
    assert_eq!(status.code(), Some(7));
}

/// A service exiting while a wait runs ends the wait at once, naming how it exited, rather than
/// after a timeout that describes the symptom.
#[test]
fn a_service_exiting_during_its_wait_fails_at_once() {
    let name = stage_script("exits_early", "exit 3");
    let util = scripts(name);
    let started = Instant::now();
    let strategy = WaitStrategy::WaitForHttpHealthCheck("http://127.0.0.1:1/".to_string(), 30);
    let res = util.start_service(name, None, &strategy, None);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "it waited out the timeout"
    );
    let Err(ServiceUtilError::ServiceExited(message)) = res else {
        panic!("expected ServiceExited: {res:?}");
    };
    assert!(message.contains('3'), "{message}");
}

/// A test can read what the service logged.
#[test]
fn stderr_goes_to_the_named_file() {
    let name = stage_script("logs", "echo to-the-log >&2");
    let util = scripts(name);
    let log = format!("{SCRIPT_ROOT}/logs.log");
    let config = ServiceStartConfig::builder()
        .program(name)
        .wait_strategy(WaitStrategy::NoWait)
        .stderr_file(&log)
        .build();
    let pid = util
        .start_service_from_config(config)
        .expect("the service must start");
    util.wait_for_exit(pid, Duration::from_secs(10))
        .expect("it must exit");
    let text = fs::read_to_string(&log).expect("the log");
    assert!(text.contains("to-the-log"), "{text}");
}

/// An unwritable log file fails the start, naming the file.
#[test]
fn a_log_file_that_cannot_be_created_is_reported() {
    let name = stage_script("no_log", "exit 0");
    let util = scripts(name);
    let config = ServiceStartConfig::builder()
        .program(name)
        .wait_strategy(WaitStrategy::NoWait)
        .stderr_file("/nonexistent/directory/service.log")
        .build();
    let Err(ServiceUtilError::ServiceStartFailed(message)) = util.start_service_from_config(config)
    else {
        panic!("expected ServiceStartFailed");
    };
    assert!(
        message.contains("/nonexistent/directory/service.log"),
        "{message}"
    );
}

/// With `clear_env` the service sees `env_vars` alone. Without it, it inherits the caller's too.
#[test]
fn clear_env_keeps_the_callers_environment_out() {
    let name = stage_script(
        "env_probe",
        "[ -z \"$HOME\" ] && [ \"$NAMED\" = yes ] && exit 10\n[ -n \"$HOME\" ] && [ \"$NAMED\" = yes ] && exit 20\nexit 1",
    );
    let util = scripts(name);
    let run = |clear_env| {
        let config = ServiceStartConfig::builder()
            .program(name)
            .wait_strategy(WaitStrategy::NoWait)
            .env_vars(vec![("NAMED".to_string(), "yes".to_string())])
            .clear_env(clear_env)
            .build();
        let pid = util
            .start_service_from_config(config)
            .expect("the service must start");
        util.wait_for_exit(pid, Duration::from_secs(10))
            .expect("it must exit")
            .code()
    };
    assert_eq!(run(true), Some(10), "the caller's HOME must not reach it");
    assert_eq!(run(false), Some(20), "it inherits by default");
}

/// Dropping the `ServiceUtil` takes what it started with it, so nothing keeps a port.
#[test]
fn dropping_the_util_stops_its_services() {
    let name = stage_script("outlives_nothing", "sleep 30");
    let util = scripts(name);
    let pid = util
        .start_service(name, None, &WaitStrategy::NoWait, None)
        .expect("the service must start");
    assert!(process_is_alive(pid));
    drop(util);
    assert!(!process_is_alive(pid), "dropping must kill and reap it");
}

/// A builder missing a required field says which, on the test's first run.
#[test]
#[should_panic(expected = "program")]
fn a_builder_without_a_program_names_it() {
    let _config = ServiceStartConfig::builder()
        .wait_strategy(WaitStrategy::NoWait)
        .build();
}

#[test]
#[should_panic(expected = "wait_strategy")]
fn a_builder_without_a_wait_strategy_names_it() {
    let _config = ServiceStartConfig::builder().program("p").build();
}

/// `new` keeps the defaults the builder has, the log and the environment untouched.
#[test]
fn a_config_defaults_to_inherited_output_and_environment() {
    let built = ServiceStartConfig::builder()
        .program("p")
        .wait_strategy(WaitStrategy::NoWait)
        .build();
    assert_eq!(built.host(), "localhost");
    assert!(built.stderr_file().is_none());
    assert!(!built.clear_env());
    let made = ServiceStartConfig::new("p", WaitStrategy::NoWait, None, None, "localhost", None);
    assert_eq!(built, made);
}

/// Whether a PID exists at all, including as a zombie.
fn process_is_alive(pid: u32) -> bool {
    process_state(pid).is_some()
}

/// Whether a PID is still a running process.
///
/// A stopped service is dead but not reaped, because the crate drops the `Child` on purpose
/// so the service can outlive the call that started it. It therefore lingers as a zombie
/// until the calling process exits, which keeps the PID in the table even though the process
/// is gone and its port released. `kill -0` cannot tell the two apart; the process state can.
fn process_is_running(pid: u32) -> bool {
    match process_state(pid) {
        Some(state) => !state.starts_with('Z'),
        None => false,
    }
}

/// The process state letter from `ps`, or `None` when the PID is gone entirely.
fn process_state(pid: u32) -> Option<String> {
    let out = std::process::Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .expect("Failed to run ps");

    let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if state.is_empty() { None } else { Some(state) }
}
