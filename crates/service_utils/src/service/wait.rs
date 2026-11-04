/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use crate::{ServiceUtil, ServiceUtilError};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;
use wait_utils::{ProbeContext, WaitStrategy};

/// How often the wait looks at whether the service is still alive.
const LIVENESS_POLL: Duration = Duration::from_millis(20);

impl ServiceUtil {
    /// Waits for the program to become ready based on the given wait strategy, and fails at once
    /// if it exits first, naming how: its readiness would otherwise only time out, far from the
    /// cause.
    pub(crate) fn wait_for_program(
        &self,
        program: &str,
        pid: u32,
        host: &'static str,
        port: Option<u16>,
        wait_strategy: WaitStrategy,
    ) -> Result<(), ServiceUtilError> {
        let (dbg, name) = (self.dbg, program.to_string());
        let (sender, receiver) = mpsc::channel();
        // A thread of its own, so this one can watch the service while a strategy blocks. On a
        // service's early exit it is left to end at its own timeout, holding nothing of worth.
        std::thread::spawn(move || {
            let _ignored = sender.send(run_strategy(dbg, &name, host, port, &wait_strategy));
        });

        loop {
            match receiver.recv_timeout(LIVENESS_POLL) {
                Ok(outcome) => return outcome,
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(ServiceUtilError::ServiceHealthcheckFailed(format!(
                        "the wait for '{program}' panicked"
                    )));
                }
                Err(RecvTimeoutError::Timeout) => {
                    if let Some(status) = self.exit_status(pid)? {
                        return Err(ServiceUtilError::ServiceExited(format!(
                            "'{program}' (PID {pid}) exited before it was ready: {status}"
                        )));
                    }
                }
            }
        }
    }
}

/// Runs `wait_strategy` to its end for the service `program`.
fn run_strategy(
    dbg: bool,
    program: &str,
    host: &'static str,
    port: Option<u16>,
    wait_strategy: &WaitStrategy,
) -> Result<(), ServiceUtilError> {
    let dbg_print = |text: &str| {
        if dbg {
            println!("[ServiceUtil]: {text}");
        }
    };
    match wait_strategy {
        WaitStrategy::WaitForDuration(duration) => {
            dbg_print(&format!(
                "[start_container]: Waiting for {duration} seconds."
            ));
            wait_utils::wait_until_timeout(duration).map_err(|e| {
                ServiceUtilError::ServiceHealthcheckFailed(format!(
                    "could not wait for {duration} seconds: {e}"
                ))
            })?;
        }

        WaitStrategy::WaitUntilConsoleOutputContains(_, _) => {
            return Err(ServiceUtilError::UnsupportedWaitStrategy(
                "WaitUntilConsoleOutputContains Strategy is not supported".into(),
            ));
        }

        WaitStrategy::WaitForHttpHealthCheck(url, duration) => {
            dbg_print(&format!(
                "[start_container]: Waiting for {:?} on HTTP health check on {}.",
                duration, url
            ));
            wait_utils::wait_until_http_health_check(dbg, url, duration).map_err(|e| {
                ServiceUtilError::ServiceHealthcheckFailed(format!(
                    "service did not pass its HTTP health check on {url}: {e}"
                ))
            })?;
        }

        #[cfg(feature = "grpc")]
        WaitStrategy::WaitForGrpcHealthCheck(url, duration) => {
            dbg_print(&format!(
                "[start_container]: Waiting for {:?} on GRPC health check on {}.",
                duration, url
            ));
            wait_utils::wait_until_grpc_health_check(dbg, url, duration).map_err(|e| {
                ServiceUtilError::ServiceHealthcheckFailed(format!(
                    "service did not pass its gRPC health check on {url}: {e}"
                ))
            })?;
        }

        // Honoured here as well as in docker_utils, so that the variant does not become
        // driver-specific: a strategy only one driver can run is the defect this crate is
        // trying not to repeat.
        //
        // The probe is synchronous by contract and may build its own async runtime, which this
        // dedicated thread guarantees it can.
        WaitStrategy::WaitUntilReady {
            probe,
            timeout_secs,
            retry_delay_ms,
        } => {
            // Reported rather than guessed. A made-up port would send the probe at the
            // wrong address and report the resulting silence as a readiness timeout,
            // which is a symptom arbitrarily far from its cause.
            let Some(port) = port else {
                return Err(ServiceUtilError::ServiceHealthcheckFailed(format!(
                    "WaitUntilReady needs an address for '{program}', but no port is set. \
                         Set .port(..) on the ServiceStartConfig and start the service with \
                         start_service_from_config."
                )));
            };

            let probe = *probe;
            let id = program.to_string();
            let host = host.to_string();
            let timeout = Duration::from_secs(*timeout_secs);
            let retry_delay = Duration::from_millis(*retry_delay_ms);

            let mut attempt = 0u32;

            let outcome = wait_utils::wait_until_ready(dbg, timeout, retry_delay, || {
                attempt = attempt.saturating_add(1);
                probe(&ProbeContext::new(id.clone(), host.clone(), port, attempt))
            });

            outcome.map_err(|e| {
                ServiceUtilError::ServiceHealthcheckFailed(format!(
                    "service '{program}' did not become ready: {e}"
                ))
            })?;
        }

        // Do nothing
        WaitStrategy::NoWait => {
            dbg_print("[start_container]: No wait. Return immediately.");
        }
    };
    Ok(())
}
