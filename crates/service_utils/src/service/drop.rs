/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use crate::ServiceUtil;

impl Drop for ServiceUtil {
    /// Kills whatever is still running and reaps it. Under Cargo there is no sandbox to do it,
    /// and a service left behind holds its port against the next run.
    fn drop(&mut self) {
        let dbg = self.dbg;
        let children = self
            .children
            .get_mut()
            .unwrap_or_else(|poison| poison.into_inner());
        for (pid, child) in children.iter_mut() {
            if let Ok(None) = child.try_wait() {
                if dbg {
                    println!("[ServiceUtil]: Killing service with PID: {pid}");
                }
                let _killed = child.kill();
                let _reaped = child.wait();
            }
        }
    }
}
