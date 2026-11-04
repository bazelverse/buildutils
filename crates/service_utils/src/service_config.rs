/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use wait_utils::WaitStrategy;

/// Create a new instance of the `ServiceStartConfig` struct using the builder.
///
/// The `program` is the name of the program to start. The `wait_strategy`
/// is the wait strategy to use to wait for the service to start. The
/// `env_var` is an optional environment variable to set when starting the
/// service.
///
/// The `host` and `port` are the address a caller would connect to. They live here rather
/// than being discovered, because the address is a chicken and egg problem: the caller cannot
/// connect until it knows the port, and the driver cannot bind until the caller has chosen
/// one. The config is what breaks it, by being the single place both sides read.
///
/// # Examples
///
/// Basic configuration using the derived builder:
/// ```rust
/// use service_utils::*;
///
/// let config = ServiceStartConfig::builder()
///     .program("program")
///     .wait_strategy(WaitStrategy::NoWait)
///     .build();
/// ```
///
/// Configuration with optional environment variables using the builder:
///
/// ```rust
/// use service_utils::*;
///
/// let config = ServiceStartConfig::builder()
///     .program("program")
///     .program_args(vec!["arg1", "arg2"])
///     .wait_strategy(WaitStrategy::NoWait)
///     .env_vars(vec![("KEY".into(), "VALUE".into())])
///     .build();
/// ```
///
/// # Returns
///
/// Returns a new `ServiceStartConfig` instance.
///
#[derive(Debug, Default, Clone, Eq, PartialOrd, Ord, PartialEq, Hash)]
pub struct ServiceStartConfig {
    program: &'static str,
    wait_strategy: WaitStrategy,
    program_args: Option<Vec<&'static str>>,
    env_vars: Option<Vec<(String, String)>>,
    /// The host a caller would connect to, handed to a readiness probe.
    host: &'static str,
    /// The port a caller would connect to, handed to a readiness probe.
    ///
    /// Optional because a service need not listen on one at all. It is required only by
    /// [`WaitStrategy::WaitUntilReady`], which reports a clear error when it is missing
    /// rather than probing a made-up address.
    port: Option<u16>,
    /// The file the service's stderr goes to, created afresh, so a test can read what it logged.
    stderr_file: Option<PathBuf>,
    /// Whether the service starts with `env_vars` alone in its environment, so nothing of the
    /// caller's own environment can reach its configuration.
    clear_env: bool,
}

/// Builds a [`ServiceStartConfig`]. `program` and `wait_strategy` are required, and `host`
/// defaults to `localhost`.
#[derive(Debug, Default, Clone)]
pub struct ServiceStartConfigBuilder {
    program: Option<&'static str>,
    wait_strategy: Option<WaitStrategy>,
    program_args: Option<Vec<&'static str>>,
    env_vars: Option<Vec<(String, String)>>,
    host: Option<&'static str>,
    port: Option<u16>,
    stderr_file: Option<PathBuf>,
    clear_env: bool,
}

impl ServiceStartConfigBuilder {
    /// The name of the program to start.
    #[must_use]
    pub fn program(mut self, program: &'static str) -> Self {
        self.program = Some(program);
        self
    }

    /// How to wait for the service to become ready.
    #[must_use]
    pub fn wait_strategy(mut self, wait_strategy: WaitStrategy) -> Self {
        self.wait_strategy = Some(wait_strategy);
        self
    }

    /// The program's arguments.
    #[must_use]
    pub fn program_args(mut self, program_args: Vec<&'static str>) -> Self {
        self.program_args = Some(program_args);
        self
    }

    /// Environment variables to set when starting the service.
    #[must_use]
    pub fn env_vars(mut self, env_vars: Vec<(String, String)>) -> Self {
        self.env_vars = Some(env_vars);
        self
    }

    /// The host a caller would connect to.
    #[must_use]
    pub fn host(mut self, host: &'static str) -> Self {
        self.host = Some(host);
        self
    }

    /// The port a caller would connect to.
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// The file the service's stderr goes to, created afresh when it starts.
    #[must_use]
    pub fn stderr_file(mut self, stderr_file: impl Into<PathBuf>) -> Self {
        self.stderr_file = Some(stderr_file.into());
        self
    }

    /// Whether the service starts with `env_vars` alone in its environment.
    #[must_use]
    pub fn clear_env(mut self, clear_env: bool) -> Self {
        self.clear_env = clear_env;
        self
    }

    /// The configuration.
    ///
    /// # Panics
    ///
    /// Panics naming `program` or `wait_strategy` when it was not set: a test written without
    /// one is a mistake in that test, found on its first run.
    pub fn build(self) -> ServiceStartConfig {
        ServiceStartConfig {
            program: self.program.expect("ServiceStartConfig needs a program"),
            wait_strategy: self
                .wait_strategy
                .expect("ServiceStartConfig needs a wait_strategy"),
            program_args: self.program_args,
            env_vars: self.env_vars,
            host: self.host.unwrap_or("localhost"),
            port: self.port,
            stderr_file: self.stderr_file,
            clear_env: self.clear_env,
        }
    }
}

impl ServiceStartConfig {
    /// A builder of a configuration.
    #[must_use]
    pub fn builder() -> ServiceStartConfigBuilder {
        ServiceStartConfigBuilder::default()
    }

    /// Create a new instance of the `ServiceStartConfig` struct using the constructor.
    ///
    /// The `program` is the name of the program to start. The `wait_strategy`
    /// is the wait strategy to use to wait for the service to start. The
    /// `env_var` is an optional environment variable to set when starting the
    /// service.
    ///
    /// # Examples
    ///
    /// Basic configuration using the constructor:
    ///
    /// ```rust
    /// use service_utils::*;
    ///
    /// let config = ServiceStartConfig::new(
    ///     "program",
    ///     WaitStrategy::NoWait,
    ///     None,
    ///     None,
    ///     "localhost",
    ///     Some(8080),
    /// );
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a new `ServiceStartConfig` instance.
    ///
    pub fn new(
        program: &'static str,
        wait_strategy: WaitStrategy,
        program_args: Option<Vec<&'static str>>,
        env_vars: Option<Vec<(String, String)>>,
        host: &'static str,
        port: Option<u16>,
    ) -> Self {
        Self {
            program,
            wait_strategy,
            program_args,
            env_vars,
            host,
            port,
            stderr_file: None,
            clear_env: false,
        }
    }
}

impl ServiceStartConfig {
    #[inline]
    pub const fn program(&self) -> &'static str {
        self.program
    }

    #[inline]
    pub const fn wait_strategy(&self) -> &WaitStrategy {
        &self.wait_strategy
    }

    #[inline]
    pub const fn env_vars(&self) -> &Option<Vec<(String, String)>> {
        &self.env_vars
    }
    #[inline]
    pub fn program_args(&self) -> &Option<Vec<&'static str>> {
        &self.program_args
    }

    /// The host a caller would connect to, handed to a readiness probe.
    #[inline]
    pub const fn host(&self) -> &'static str {
        self.host
    }

    /// The port a caller would connect to, handed to a readiness probe.
    #[inline]
    pub const fn port(&self) -> Option<u16> {
        self.port
    }

    /// The file the service's stderr goes to, if one was named.
    #[inline]
    pub fn stderr_file(&self) -> Option<&Path> {
        self.stderr_file.as_deref()
    }

    /// Whether the service starts with `env_vars` alone in its environment.
    #[inline]
    pub const fn clear_env(&self) -> bool {
        self.clear_env
    }
}

impl Display for ServiceStartConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ServiceStartConfig {{ program: {}, wait_strategy: {}, env_vars: {:?}, \
            host: {}, port: {:?} }}",
            self.program, self.wait_strategy, self.env_vars, self.host, self.port
        )
    }
}
