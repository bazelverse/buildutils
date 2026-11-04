/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use crate::WaitStrategyError;
use std::time::{Duration, Instant};
use tonic::transport::Channel;
use tonic_health::pb::HealthCheckRequest;
use tonic_health::pb::health_client::HealthClient;

/// Waits until a gRPC service at the given `health_url` is online by performing
/// a health check. The health check is performed every 100ms until the service
/// responds or the given `timeout` is reached. If the timeout is reached without
/// a response, a `ServiceUtilError` is returned.
///
/// The health check is done by calling the `Check` method on the gRPC health
/// service. The `service` parameter of the `HealthCheckRequest` is an empty string.
///
///
/// The check runs on a private runtime on its own thread, so a caller stays synchronous and
/// may call this from anywhere, inside an async runtime of its own included.
///
pub fn wait_until_grpc_health_check(
    dbg: bool,
    health_url: &str,
    timeout: &u64,
) -> Result<(), WaitStrategyError> {
    let (url, timeout) = (health_url.to_string(), *timeout);
    let check = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                WaitStrategyError(format!(
                    "[wait_until_grpc_health_check]: could not build its runtime: {e}"
                ))
            })?
            .block_on(check_until_healthy(dbg, &url, timeout))
    });
    check.join().map_err(|_| {
        WaitStrategyError("[wait_until_grpc_health_check]: the check panicked".to_string())
    })?
}

async fn check_until_healthy(
    dbg: bool,
    health_url: &str,
    timeout: u64,
) -> Result<(), WaitStrategyError> {
    let endpoint = Channel::from_shared(health_url.to_string()).map_err(|e| {
        WaitStrategyError(format!(
            "[wait_until_grpc_health_check]: {health_url} is no valid URI: {e}"
        ))
    })?;
    let start_time = Instant::now();
    let timeout = Duration::from_secs(timeout);

    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;

        if start_time.elapsed().as_secs() > timeout.as_secs() {
            return Err(WaitStrategyError(format!(
                "[wait_until_grpc_health_check]: !!Timeout!! Waited {} seconds for service health check to pass.",
                timeout.as_secs(),
            )));
        }

        match endpoint.connect().await {
            Ok(channel) => {
                let mut client = HealthClient::new(channel);
                let request = tonic::Request::new(HealthCheckRequest {
                    service: "".to_string(), // Default setting. Works for all services.
                });

                match client.check(request).await {
                    Ok(_) => {
                        if dbg {
                            println!("gRPC Service online");
                        }
                        break Ok(());
                    }
                    Err(e) => {
                        println!("[wait_until_grpc_health_check]: Health check failed: {e}");
                    }
                }
            }
            Err(e) => {
                println!("[wait_until_grpc_health_check]: Connection failed: {e}");
            }
        }
    }
}
