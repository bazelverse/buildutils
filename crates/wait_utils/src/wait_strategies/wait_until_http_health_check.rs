/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use crate::WaitStrategyError;
use crate::utils::utils_test::{HttpTarget, http_status, parse_http_url};
use std::time::{Duration, Instant};

/// How long one probe may take: a listener that accepts and says nothing is not ready.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Waits until the health check URL responds with a success status.
///
/// Only a status below 400 counts as ready. A service that is already listening but reports
/// itself as not ready, typically with a 503, keeps the wait going rather than ending it.
///
/// The check is plain HTTP/1.1 over `std`, so it needs no tool on the host and no runtime.
///
/// # Arguments
///
/// * `health_url` - The URL to ping for health check, `http://host[:port][/path]`.
///
/// # Returns
///
/// Returns a `WaitStrategyError` if the healthcheck times out, and at once for a URL it
/// cannot probe, such as an `https://` one, since waiting could not change that.
///
pub fn wait_until_http_health_check(
    dbg: bool,
    health_url: &str,
    timeout: &u64,
) -> Result<(), WaitStrategyError> {
    let target = parse_http_url(health_url)
        .map_err(|e| WaitStrategyError(format!("[wait_until_http_health_check]: {e}")))?;
    let start_time = Instant::now();
    let timeout = Duration::from_secs(*timeout);

    loop {
        std::thread::sleep(Duration::from_millis(100));

        if start_time.elapsed().as_secs() > timeout.as_secs() {
            return Err(WaitStrategyError(format!(
                "[wait_until_http_health_check]: !!Timeout!! Waited {} seconds for service health check to pass.",
                timeout.as_secs(),
            )));
        }

        if probe(dbg, &target) {
            if dbg {
                println!("Service online");
            }
            return Ok(());
        }
    }
}

/// One-shot form of the HTTP health check: does the URL answer with a success status?
///
/// Exposed so a driver can compose the check with its own retry loop and keep checking
/// whatever else it knows about, such as whether the container is still alive. The looping
/// [`wait_until_http_health_check`] cannot do that, because it knows nothing but the URL.
///
/// # Arguments
///
/// * `dbg` - Whether to print the outcome of the check.
/// * `health_url` - The URL to ping for health check.
///
/// # Returns
///
/// Returns whether the endpoint answered with a status below 400. A URL that cannot be
/// probed, or any failure to reach the endpoint, counts as not ready.
///
pub fn http_check_ok(dbg: bool, health_url: &str) -> bool {
    match parse_http_url(health_url) {
        Ok(target) => probe(dbg, &target),
        Err(e) => {
            if dbg {
                println!("[http_check_ok]: {e}");
            }
            false
        }
    }
}

fn probe(dbg: bool, target: &HttpTarget) -> bool {
    match http_status(target, PROBE_TIMEOUT) {
        Ok(status) if status < 400 => true,
        Ok(status) => {
            if dbg {
                println!(
                    "[http_check]: {} not ready: status {status}",
                    target.authority()
                );
            }
            false
        }
        Err(e) => {
            if dbg {
                println!("[http_check]: {} not ready: {e}", target.authority());
            }
            false
        }
    }
}
