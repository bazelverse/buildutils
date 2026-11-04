/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use std::fmt::Display;
use wait_utils::WaitStrategy;
// https://github.com/elastio/bon

/// Create a new instance of the `ContainerConfig` struct with the given parameters.
///
/// Examples
///
/// Creating a new configuration using the derived builder:
/// ```rust
///  use docker_utils::*;
///
///     let config = ContainerConfig::builder()
///         .name("test_container")
///         .image("test_image")
///         .tag("latest")
///         .url("0.0.0.0")
///         .connection_port(8080)
///         .reuse_container(true)
///         .keep_configuration(true)
///         .wait_strategy(WaitStrategy::NoWait)
///         .build();
/// ```
///
/// Mounting a file into the container, for a program that takes a path rather than a value:
/// ```rust
///  use docker_utils::*;
///
///     let config = ContainerConfig::builder()
///         .name("test_container")
///         .image("test_image")
///         .tag("latest")
///         .url("0.0.0.0")
///         .connection_port(8080)
///         .reuse_container(true)
///         .keep_configuration(true)
///         .volumes(&["/host/secrets:/run/secrets:ro"])
///         .wait_strategy(WaitStrategy::NoWait)
///         .build();
/// ```
///
/// Creating a configuration that runs the container on the host network:
/// ```rust
///  use docker_utils::*;
///
///     let config = ContainerConfig::builder()
///         .name("test_container")
///         .image("test_image")
///         .tag("latest")
///         .url("0.0.0.0")
///         .connection_port(8080)
///         .reuse_container(true)
///         .keep_configuration(true)
///         .host_network(true)
///         .wait_strategy(WaitStrategy::NoWait)
///         .build();
/// ```
///
#[derive(Debug, Default, Clone, Eq, PartialOrd, Ord, PartialEq, Hash)]
pub struct ContainerConfig<'l> {
    name: &'l str,
    image: &'l str,
    tag: &'l str,
    url: &'l str,
    connection_port: u16,
    reuse_container: bool,
    keep_configuration: bool,
    additional_ports: Option<&'l [u16]>,
    additional_env_vars: Option<&'l [&'l str]>,
    /// Bind mounts, each in Docker's `-v` form: `host:container[:ro]`.
    ///
    /// The reason this exists rather than callers baking files into an image: some programs
    /// take a path, not a value. Dgraph's `--acl "secret-file=<path>"` is the case that
    /// prompted it -- there is no inline form, so a secret can only reach the container as a
    /// file, and putting it in an image would commit it to a layer.
    volumes: Option<&'l [&'l str]>,
    platform: Option<&'l str>,
    /// Run the container on the host network instead of publishing ports.
    ///
    /// Optional in the builder and defaults to `false`, so existing configurations
    /// keep publishing ports exactly as before.
    host_network: bool,
    wait_strategy: WaitStrategy,
}

/// Builds a [`ContainerConfig`]. Every field is required but the optional ones and `host_network`,
/// which defaults to `false`.
#[derive(Debug, Default, Clone)]
pub struct ContainerConfigBuilder<'l> {
    name: Option<&'l str>,
    image: Option<&'l str>,
    tag: Option<&'l str>,
    url: Option<&'l str>,
    connection_port: Option<u16>,
    reuse_container: Option<bool>,
    keep_configuration: Option<bool>,
    additional_ports: Option<&'l [u16]>,
    additional_env_vars: Option<&'l [&'l str]>,
    volumes: Option<&'l [&'l str]>,
    platform: Option<&'l str>,
    host_network: bool,
    wait_strategy: Option<WaitStrategy>,
}

impl<'l> ContainerConfigBuilder<'l> {
    /// The name of the container.
    #[must_use]
    pub fn name(mut self, name: &'l str) -> Self {
        self.name = Some(name);
        self
    }

    /// The image to use for the container.
    #[must_use]
    pub fn image(mut self, image: &'l str) -> Self {
        self.image = Some(image);
        self
    }

    /// The tag of the image.
    #[must_use]
    pub fn tag(mut self, tag: &'l str) -> Self {
        self.tag = Some(tag);
        self
    }

    /// The default URL of the container.
    #[must_use]
    pub fn url(mut self, url: &'l str) -> Self {
        self.url = Some(url);
        self
    }

    /// The port number of the main connection.
    #[must_use]
    pub fn connection_port(mut self, connection_port: u16) -> Self {
        self.connection_port = Some(connection_port);
        self
    }

    /// Whether to reuse an existing container.
    #[must_use]
    pub fn reuse_container(mut self, reuse_container: bool) -> Self {
        self.reuse_container = Some(reuse_container);
        self
    }

    /// Whether to keep the configuration.
    #[must_use]
    pub fn keep_configuration(mut self, keep_configuration: bool) -> Self {
        self.keep_configuration = Some(keep_configuration);
        self
    }

    /// Additional ports to publish.
    #[must_use]
    pub fn additional_ports(mut self, additional_ports: &'l [u16]) -> Self {
        self.additional_ports = Some(additional_ports);
        self
    }

    /// Additional environment variables.
    #[must_use]
    pub fn additional_env_vars(mut self, additional_env_vars: &'l [&'l str]) -> Self {
        self.additional_env_vars = Some(additional_env_vars);
        self
    }

    /// Bind mounts, each in Docker's `-v` form.
    #[must_use]
    pub fn volumes(mut self, volumes: &'l [&'l str]) -> Self {
        self.volumes = Some(volumes);
        self
    }

    /// The platform, for an image that is not multi-arch.
    #[must_use]
    pub fn platform(mut self, platform: &'l str) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Runs the container on the host network instead of publishing ports.
    #[must_use]
    pub fn host_network(mut self, host_network: bool) -> Self {
        self.host_network = host_network;
        self
    }

    /// How to wait for the container to become ready.
    #[must_use]
    pub fn wait_strategy(mut self, wait_strategy: WaitStrategy) -> Self {
        self.wait_strategy = Some(wait_strategy);
        self
    }

    /// The configuration.
    ///
    /// # Panics
    ///
    /// Panics naming the required field that was not set: a test written without one is a
    /// mistake in that test, found on its first run.
    pub fn build(self) -> ContainerConfig<'l> {
        ContainerConfig {
            name: self.name.expect("ContainerConfig needs a name"),
            image: self.image.expect("ContainerConfig needs a image"),
            tag: self.tag.expect("ContainerConfig needs a tag"),
            url: self.url.expect("ContainerConfig needs a url"),
            connection_port: self
                .connection_port
                .expect("ContainerConfig needs a connection_port"),
            reuse_container: self
                .reuse_container
                .expect("ContainerConfig needs a reuse_container"),
            keep_configuration: self
                .keep_configuration
                .expect("ContainerConfig needs a keep_configuration"),
            additional_ports: self.additional_ports,
            additional_env_vars: self.additional_env_vars,
            volumes: self.volumes,
            platform: self.platform,
            host_network: self.host_network,
            wait_strategy: self
                .wait_strategy
                .expect("ContainerConfig needs a wait_strategy"),
        }
    }
}

impl<'l> ContainerConfig<'l> {
    /// A builder of a configuration.
    #[must_use]
    pub fn builder() -> ContainerConfigBuilder<'l> {
        ContainerConfigBuilder::default()
    }

    /// Create a new instance of the `ContainerConfig` struct with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the container.
    /// * `image` - The image to use for the container.
    /// * `tag` - The tag of the image.
    /// * `url` - The default URL of the container. Usually 0.0.0.0
    /// * `connection_port` - The port number for the main connection i.e. 80 for a webserver.
    /// * `additional_ports` - An optional array of additional ports to publish.
    /// * `volumes` - Optional bind mounts, each in Docker's `-v` form: `host:container[:ro]`.
    /// * `platform` - An optional platform string in case the container image is not multi-arch.
    /// * `host_network` - Run the container on the host network instead of publishing ports.
    ///   When true, no port is published because Docker discards published ports in host
    ///   network mode.
    /// * `reuse_container` - A boolean flag indicating whether to reuse an existing container if found.
    /// * `keep_configuration` -  A boolean flag indication whether to keep the configuration upon
    ///   every environment setup. If set to true, the same configuration will be used across all
    ///   environment setups. If false, each setup will re-create all tables and import data.,
    /// * `wait_strategy` - The wait strategy to use for the container.
    ///
    /// Examples
    ///
    /// Creating a new configuration using the constructor:
    /// ```rust
    /// use docker_utils::*;
    ///
    ///     let config =  ContainerConfig::new(
    ///         "test_container",
    ///         "test_image",
    ///         "latest",
    ///         "0.0.0.0",
    ///         8080,
    ///         Some(&[8081, 8082]),
    ///         Some(&["ENV_VAR=VALUE", "DEBUG=true"]),
    ///         Some(&["/etc/secrets:/run/secrets:ro"]),
    ///         Some("linux/amd64"),
    ///         false, // Publish ports rather than using the host network
    ///         true,
    ///         false,
    ///         WaitStrategy::default(), // NoWait is the default wait strategy
    ///     );
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a new instance of the `ContainerConfig` struct.
    ///
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        name: &'l str,
        image: &'l str,
        tag: &'l str,
        url: &'l str,
        connection_port: u16,
        additional_ports: Option<&'l [u16]>,
        additional_env_vars: Option<&'l [&'l str]>,
        volumes: Option<&'l [&'l str]>,
        platform: Option<&'l str>,
        host_network: bool,
        reuse_container: bool,
        keep_configuration: bool,
        wait_strategy: WaitStrategy,
    ) -> Self {
        Self {
            name,
            image,
            tag,
            url,
            connection_port,
            reuse_container,
            keep_configuration,
            additional_ports,
            additional_env_vars,
            volumes,
            platform,
            host_network,
            wait_strategy,
        }
    }
}

impl<'l> ContainerConfig<'l> {
    #[inline]
    pub const fn name(&self) -> &'l str {
        self.name
    }
    #[inline]
    pub fn container_image(&self) -> String {
        format!("{}:{}", self.image, self.tag)
    }
    #[inline]
    pub fn container_name(&self) -> String {
        format!("{}-{}", self.name, self.connection_port)
    }
    #[inline]
    pub const fn url(&self) -> &'l str {
        self.url
    }
    #[inline]
    pub const fn connection_port(&self) -> u16 {
        self.connection_port
    }
    #[inline]
    pub const fn additional_ports(&self) -> Option<&'l [u16]> {
        self.additional_ports
    }
    #[inline]
    pub const fn additional_env_vars(&self) -> Option<&'l [&'l str]> {
        self.additional_env_vars
    }
    /// Bind mounts, each in Docker's `-v` form: `host:container[:ro]`.
    #[inline]
    pub const fn volumes(&self) -> Option<&'l [&'l str]> {
        self.volumes
    }
    #[inline]
    pub const fn platform(&self) -> Option<&'l str> {
        self.platform
    }
    #[inline]
    pub const fn reuse_container(&self) -> bool {
        self.reuse_container
    }
    #[inline]
    pub const fn keep_configuration(&self) -> bool {
        self.keep_configuration
    }
    /// Whether the container runs on the host network.
    ///
    /// When true, the container is started with `--network host` and no ports are
    /// published, because Docker discards published ports in host network mode.
    #[inline]
    pub const fn host_network(&self) -> bool {
        self.host_network
    }
    #[inline]
    pub const fn wait_strategy(&self) -> &WaitStrategy {
        &self.wait_strategy
    }
    #[inline]
    pub const fn image(&self) -> &'l str {
        self.image
    }
    #[inline]
    pub const fn tag(&self) -> &'l str {
        self.tag
    }
}

impl Display for ContainerConfig<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name: {}, image: {}:{}, url: {} connection_port: {}, additional_ports: {:?}, \
            additional_env_vars: {:?}, platform: {:?},  reuse_container: {}, keep_configuration: {}, \
            host_network: {}, wait_strategy: {}",
            self.name,
            self.image,
            self.tag,
            self.url,
            self.connection_port,
            self.additional_ports,
            self.additional_env_vars,
            self.platform,
            self.reuse_container,
            self.keep_configuration,
            self.host_network,
            self.wait_strategy,
        )
    }
}
