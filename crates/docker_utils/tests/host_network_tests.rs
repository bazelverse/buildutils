/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use docker_utils::{ContainerConfig, DockerUtil, WaitStrategy};
use std::process::Command;

// Redis is used rather than the nginx container of the other integration tests, so that a
// host network container binding the host's ports directly cannot collide with them.
const HOST_NETWORK_NAME: &str = "docker-utils-hostnet";
const HOST_NETWORK_PORT: u16 = 6379;

fn get_host_network_container_config() -> ContainerConfig<'static> {
    ContainerConfig::builder()
        .name(HOST_NETWORK_NAME)
        .image("redis")
        .tag("7-alpine")
        .url("0.0.0.0")
        .connection_port(HOST_NETWORK_PORT)
        .host_network(true)
        .reuse_container(false)
        .keep_configuration(false)
        .wait_strategy(WaitStrategy::NoWait)
        .build()
}

/// Reads a single Go template field from `docker inspect`.
fn inspect(container_id: &str, format: &str) -> String {
    let out = Command::new("docker")
        .arg("inspect")
        .arg(format!("--format={format}"))
        .arg(container_id)
        .output()
        .expect("Failed to run docker inspect");

    assert!(
        out.status.success(),
        "docker inspect failed for {container_id}: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A container configured with `host_network` must run on the host network and publish
/// nothing, because Docker discards published ports in host network mode.
#[test]
fn test_host_network_container() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let container_config = get_host_network_container_config();
    let container_id = format!("{HOST_NETWORK_NAME}-{HOST_NETWORK_PORT}");

    let res = docker_util.get_or_start_container(&container_config);
    assert!(
        res.is_ok(),
        "Failed to start host network container: {res:?}"
    );

    let (started_id, started_port) = res.unwrap();
    assert_eq!(started_id, container_id);
    assert_eq!(started_port, HOST_NETWORK_PORT);

    // Read the container state before cleaning up, so that a failed assertion below cannot
    // leave the container behind.
    let network_mode = inspect(&container_id, "{{.HostConfig.NetworkMode}}");
    let port_bindings = inspect(&container_id, "{{.HostConfig.PortBindings}}");
    let is_running = docker_util.check_if_container_is_running(&container_id);

    docker_util
        .stop_container(&container_id, true)
        .expect("Failed to stop the host network container");

    assert!(
        is_running.expect("Failed to check if the container is running"),
        "The host network container must be running"
    );
    assert_eq!(
        network_mode, "host",
        "The container must run on the host network"
    );
    assert_eq!(
        port_bindings, "map[]",
        "Host network mode must not publish any port, got: {port_bindings}"
    );
}

/// The same container published rather than on the host network, to confirm that the
/// assertions above actually distinguish the two modes.
#[test]
fn test_published_container_is_not_on_the_host_network() {
    let docker_util = DockerUtil::new().expect("Failed to create DockerUtil");
    let name = "docker-utils-published";
    let port = 6380;
    let container_id = format!("{name}-{port}");

    let container_config = ContainerConfig::builder()
        .name(name)
        .image("redis")
        .tag("7-alpine")
        .url("0.0.0.0")
        .connection_port(port)
        .host_network(false)
        .reuse_container(false)
        .keep_configuration(false)
        .wait_strategy(WaitStrategy::NoWait)
        .build();

    let res = docker_util.get_or_start_container(&container_config);
    assert!(res.is_ok(), "Failed to start published container: {res:?}");

    let network_mode = inspect(&container_id, "{{.HostConfig.NetworkMode}}");
    let port_bindings = inspect(&container_id, "{{.HostConfig.PortBindings}}");

    docker_util
        .stop_container(&container_id, true)
        .expect("Failed to stop the published container");

    assert_ne!(network_mode, "host");
    assert!(
        port_bindings.contains(&port.to_string()),
        "The connection port must be published, got: {port_bindings}"
    );
}
