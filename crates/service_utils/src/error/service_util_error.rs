/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use std::error::Error;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ServiceUtilError {
    BinaryNotFound(String),
    ServiceStartFailed(String),
    ServiceAlreadyRunning(String),
    ServiceHealthcheckFailed(String),
    ServiceStopFailed(String),
    ServiceNotSupported(String),
    ServiceNotRunning(String),
    /// The service exited before it became ready. The message holds how it exited and its PID,
    /// which `wait_for_exit` still answers for.
    ServiceExited(String),
    UnsupportedWaitStrategy(String),
    UnknownError(String),
}

impl Error for ServiceUtilError {}

impl fmt::Display for ServiceUtilError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BinaryNotFound(e) => {
                write!(f, "[ServiceUtilError]: Binary not found: {e}")
            }
            Self::ServiceStartFailed(e) => {
                write!(f, "[ServiceUtilError]: Service start failed: {e}")
            }
            Self::ServiceAlreadyRunning(e) => {
                write!(f, "[ServiceUtilError]: Service already running: {e}")
            }
            Self::ServiceHealthcheckFailed(e) => {
                write!(f, "[ServiceUtilError]: Service healthcheck failed: {e}")
            }
            Self::ServiceStopFailed(e) => {
                write!(f, "[ServiceUtilError]: Service stop failed: {e}")
            }
            Self::ServiceNotSupported(e) => {
                write!(f, "[ServiceUtilError]: Service not supported: {e}")
            }
            Self::ServiceNotRunning(e) => {
                write!(f, "[ServiceUtilError]: Service not running: {e}")
            }
            Self::ServiceExited(e) => {
                write!(f, "[ServiceUtilError]: Service exited: {e}")
            }
            Self::UnsupportedWaitStrategy(e) => {
                write!(f, "[ServiceUtilError]: Unsupported wait strategy: {e}")
            }
            Self::UnknownError(e) => {
                write!(f, "[ServiceUtilError]: Unknown error: {e}")
            }
        }
    }
}
