/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use crate::{ServiceStartConfig, ServiceUtil, ServiceUtilError};
use std::process::ExitStatus;
use std::time::Duration;
use wait_utils::WaitStrategy;

impl ServiceUtil {
    /// Creates a new ServiceUtil instance.
    ///
    /// The `root_path` is the absolute path to the root directory of the
    /// service binaries. The `binaries` is a vector of names of the binaries
    /// that should be found in the `root_path`. The constructor checks if
    /// if each binary exists in the `root_path`.
    ///
    /// # Errors
    ///
    /// Fails if any of the binaries are not found in the `root_path`.
    ///
    pub fn new(
        root_path: &'static str,
        binaries: Vec<&'static str>,
    ) -> Result<Self, ServiceUtilError> {
        Self::build(false, root_path, binaries)
    }

    /// Creates a new ServiceUtil instance with debug mode.
    ///
    /// The `root_path` is the absolute path to the root directory of the
    /// service binaries. The `binaries` is a vector of names of the binaries
    /// that should be found in the `root_path`. The constructor checks if
    /// if each binary exists in the `root_path`.
    ///
    /// # Errors
    ///
    /// Fails if any of the binaries are not found in the `root_path`.
    ///
    pub fn with_debug(
        root_path: &'static str,
        binaries: Vec<&'static str>,
    ) -> Result<Self, ServiceUtilError> {
        Self::build(true, root_path, binaries)
    }

    /// Starts a service.
    ///
    /// The `program` is the name of the program to start. The `wait_strategy`
    /// is the wait strategy to use to wait for the service to start. The
    /// `env_var` is an optional environment variable to set when starting the
    /// service.
    ///
    /// This form carries no address, so it cannot honour
    /// [`WaitStrategy::WaitUntilReady`], whose probe needs one. Use
    /// [`ServiceUtil::start_service_from_config`] with a `ServiceStartConfig` for that: the
    /// config is where the address lives, because it is the one place the caller and the
    /// driver both read it.
    ///
    /// # Returns
    ///
    /// Returns the process ID of the started service, for use with
    /// [`ServiceUtil::stop_service`].
    ///
    /// # Errors
    ///
    /// Fails if the service fails to start.
    ///
    pub fn start_service(
        &self,
        program: &str,
        program_args: Option<Vec<&str>>,
        wait_strategy: &WaitStrategy,
        env_vars: Option<Vec<(String, String)>>,
    ) -> Result<u32, ServiceUtilError> {
        self.start(
            program,
            program_args,
            env_vars,
            "localhost",
            None,
            wait_strategy.to_owned(),
            None,
            false,
        )
    }

    /// Starts a service with the given configuration.
    ///
    /// The `config` is the configuration of the service to start. The
    /// `wait_strategy` is the wait strategy to use to wait for the service to
    /// start. The `env_var` is an optional environment variable to set when
    /// starting the service.
    ///
    /// # Errors
    ///
    /// Fails if the service fails to start.
    ///
    pub fn start_service_from_config(
        &self,
        service_start_config: ServiceStartConfig,
    ) -> Result<u32, ServiceUtilError> {
        // see src/service/start.rs
        self.start_config(service_start_config)
    }

    /// Stops a service previously started by this `ServiceUtil`, and tells how it exited.
    ///
    /// Sends SIGTERM and waits for the exit, so a test can check that a service shuts down
    /// cleanly. A service still running when this `ServiceUtil` is dropped is killed then.
    ///
    /// # Arguments
    ///
    /// * `pid` - The process ID returned by `start_service` or `start_service_from_config`.
    ///
    /// # Returns
    ///
    /// Returns the service's exit status, including when it had already exited.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use service_utils::{ServiceUtil, WaitStrategy};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let service_util = ServiceUtil::new("bin", vec!["my_service"])?;
    ///
    /// let pid = service_util.start_service("my_service", None, &WaitStrategy::NoWait, None)?;
    ///
    /// let status = service_util.stop_service(pid)?;
    /// assert!(status.success());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Fails if this `ServiceUtil` did not start `pid`, it could not be signalled, or it did not
    /// exit.
    ///
    pub fn stop_service(&self, pid: u32) -> Result<ExitStatus, ServiceUtilError> {
        // see src/service/stop.rs
        self.stop(pid)
    }

    /// Waits up to `timeout` for a started service to exit by itself, and tells how it did.
    ///
    /// This is how a test checks a service that must refuse to start: start it with
    /// [`WaitStrategy::NoWait`], then read its exit code here.
    ///
    /// # Errors
    ///
    /// Fails if this `ServiceUtil` did not start `pid`, or it is still running after `timeout`.
    ///
    pub fn wait_for_exit(
        &self,
        pid: u32,
        timeout: Duration,
    ) -> Result<ExitStatus, ServiceUtilError> {
        // see src/service/stop.rs
        self.wait_exit(pid, timeout)
    }
}
