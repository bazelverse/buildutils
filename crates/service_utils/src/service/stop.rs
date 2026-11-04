/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use crate::{ServiceUtil, ServiceUtilError};
use std::collections::HashMap;
use std::process::{Child, Command, ExitStatus};
use std::sync::MutexGuard;
use std::time::{Duration, Instant};

/// How long a stopped service has to exit. A clean shutdown may flush what it holds.
const STOP_TIMEOUT: Duration = Duration::from_secs(30);
const EXIT_POLL: Duration = Duration::from_millis(20);

impl ServiceUtil {
    /// Sends SIGTERM to a started service and waits for it to exit. One that has already exited
    /// is not signalled, since its PID may have been reused.
    pub(crate) fn stop(&self, pid: u32) -> Result<ExitStatus, ServiceUtilError> {
        self.dbg_print(&format!("[stop_service]: Stopping service with PID: {pid}"));

        if let Some(status) = self.exit_status(pid)? {
            self.dbg_print(&format!("[stop_service]: PID {pid} had already exited."));
            return Ok(status);
        }

        let out = Command::new("kill")
            .arg(pid.to_string())
            .output()
            .map_err(|e| {
                ServiceUtilError::ServiceStopFailed(format!("could not run kill for {pid}: {e}"))
            })?;

        if !out.status.success() {
            // It exited between the check and the signal, which is the requested end state.
            if let Some(status) = self.exit_status(pid)? {
                return Ok(status);
            }
            return Err(ServiceUtilError::ServiceStopFailed(format!(
                "could not stop the service with PID {pid}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }

        self.wait_exit(pid, STOP_TIMEOUT).map_err(|e| {
            ServiceUtilError::ServiceStopFailed(format!("after SIGTERM to {pid}: {e}"))
        })
    }

    /// Waits up to `timeout` for a started service to exit by itself.
    pub(crate) fn wait_exit(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<ExitStatus, ServiceUtilError> {
        let started = Instant::now();
        loop {
            if let Some(status) = self.exit_status(pid)? {
                return Ok(status);
            }
            if started.elapsed() >= timeout {
                return Err(ServiceUtilError::ServiceNotRunning(format!(
                    "the service with PID {pid} is still running after {timeout:?}"
                )));
            }
            std::thread::sleep(EXIT_POLL);
        }
    }

    /// How a started service exited; `None` while it runs. Its status stays readable afterwards.
    pub(crate) fn exit_status(&self, pid: u32) -> Result<Option<ExitStatus>, ServiceUtilError> {
        let mut children = self.children();
        let child = children.get_mut(&pid).ok_or_else(|| {
            ServiceUtilError::ServiceNotRunning(format!("this ServiceUtil started no PID {pid}"))
        })?;
        child.try_wait().map_err(|e| {
            ServiceUtilError::ServiceStopFailed(format!("could not read PID {pid}'s status: {e}"))
        })
    }

    pub(crate) fn children(&self) -> MutexGuard<'_, HashMap<u32, Child>> {
        self.children
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
