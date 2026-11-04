/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

mod build;
mod dbg;
mod display;
mod drop;
mod getters;
mod start;
mod stop;
mod verify;
mod wait;

use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;

/// Starts services and owns them: each stays running until it is stopped or this is dropped, so
/// a test cannot leave one holding its port, and its exit status can always be read.
#[derive(Debug)]
pub struct ServiceUtil {
    dbg: bool,
    root_path: &'static str, // root_path basically remains constant after initialization
    binaries: Vec<&'static str>, // After verification, we're only reading from the Vec, thus lock-free
    /// Every service started, by PID. `start` takes `&self` and a test may share this.
    children: Mutex<HashMap<u32, Child>>,
}
