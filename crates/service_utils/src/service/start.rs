/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use crate::{ServiceStartConfig, ServiceUtil, ServiceUtilError};
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use wait_utils::WaitStrategy;

impl ServiceUtil {
    pub(crate) fn start_config(
        &self,
        service_start_config: ServiceStartConfig,
    ) -> Result<u32, ServiceUtilError> {
        // Extract parameters
        let program = service_start_config.program();
        let program_args = service_start_config.program_args().to_owned();
        let wait_strategy = service_start_config.wait_strategy().to_owned();
        let env_vars = service_start_config.env_vars().to_owned();
        let host = service_start_config.host();
        let port = service_start_config.port();

        // Start the service
        self.start(
            program,
            program_args,
            env_vars,
            host,
            port,
            wait_strategy,
            service_start_config.stderr_file(),
            service_start_config.clear_env(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start(
        &self,
        program: &str,
        program_args: Option<Vec<&str>>,
        env_vars: Option<Vec<(String, String)>>,
        host: &'static str,
        port: Option<u16>,
        wait_strategy: WaitStrategy,
        stderr_file: Option<&Path>,
        clear_env: bool,
    ) -> Result<u32, ServiceUtilError> {
        // Check if the program is in the binaries vector
        if !self.binaries().contains(&program) {
            return Err(ServiceUtilError::BinaryNotFound(format!(
                "Binary has not been added to the ServiceUtil. \
             Please add the following binary to the set of programs when constructing the ServiceUtil: {}",
                program
            )));
        }

        // Check if the binary in the full path still exists
        let bin = format!("{}/{}", self.root_path(), program);
        if !std::path::Path::new(&bin).exists() {
            return Err(ServiceUtilError::BinaryNotFound(format!(
                "Program {} not found in path: {}",
                program, bin
            )));
        }

        self.dbg_print(" Set the program to be executable");
        Command::new("chmod")
            .arg("+x")
            .arg(&bin)
            .output()
            .map_err(|e| {
                ServiceUtilError::ServiceStartFailed(format!("could not run chmod: {e}"))
            })?;

        self.dbg_print("Constructing start command");
        let mut cmd = Command::new(bin);

        if clear_env {
            cmd.env_clear();
        }

        if let Some(file) = stderr_file {
            let stderr = File::create(file).map_err(|e| {
                ServiceUtilError::ServiceStartFailed(format!(
                    "could not create {} for '{program}': {e}",
                    file.display()
                ))
            })?;
            cmd.stderr(Stdio::from(stderr));
        }

        if let Some(env_vars) = env_vars {
            self.dbg_print("Setting environment variables");

            // Add environment variables
            cmd.envs(env_vars);
        }

        if let Some(program_args) = program_args {
            self.dbg_print("Setting program arguments");

            // Add program arguments
            cmd.args(program_args);
        }

        self.dbg_print(&format!("Run start command: {:?}", cmd));

        let child = cmd.spawn().map_err(|e| {
            ServiceUtilError::ServiceStartFailed(format!("could not start '{program}': {e}"))
        })?;
        let pid = child.id();
        let _previous = self.children().insert(pid, child);

        self.dbg_print(&format!("Waiting for service to start. PID: {pid}"));
        self.wait_for_program(program, pid, host, port, wait_strategy)?;

        self.dbg_print("Service started");

        Ok(pid)
    }
}
