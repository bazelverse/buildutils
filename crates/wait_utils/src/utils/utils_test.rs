/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

//! Helpers behind the wait strategies.
//!
//! They are public so that the test suite under `tests/` can exercise them directly. None
//! runs a command, so a strategy built on them needs no tool on the host.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// The most bytes of a response read looking for its status line.
const STATUS_LINE_MAX: usize = 4096;

/// Where an `http://` URL points: the host and port to connect to, and the path to ask for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpTarget {
    host: String,
    port: u16,
    path: String,
}

impl HttpTarget {
    /// The host to connect to.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The port to connect to, 80 when the URL names none.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The path to ask for, `/` when the URL names none.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The target as `host:port`.
    pub fn authority(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Reads an `http://host[:port][/path]` URL.
///
/// A bracketed IPv6 host keeps its brackets out of [`HttpTarget::host`]. An `https://` URL is
/// refused by name: verifying a certificate takes a TLS stack, which this crate does not carry.
pub fn parse_http_url(url: &str) -> Result<HttpTarget, String> {
    if url.starts_with("https://") {
        return Err(format!(
            "{url}: https is not supported, since a health check here speaks plain HTTP"
        ));
    }
    let Some(rest) = url.strip_prefix("http://") else {
        return Err(format!("{url}: only http:// URLs can be probed"));
    };
    let (authority, path) = match rest.find('/') {
        Some(at) => rest.split_at(at),
        None => (rest, "/"),
    };
    let (host, port) = if let Some(bracketed) = authority.strip_prefix('[') {
        let (host, tail) = bracketed
            .split_once(']')
            .ok_or_else(|| format!("{url}: an IPv6 host's bracket is not closed"))?;
        (host, tail.strip_prefix(':'))
    } else {
        match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        }
    };
    if host.is_empty() {
        return Err(format!("{url}: the URL names no host"));
    }
    let port = match port {
        Some(port) => port
            .parse()
            .map_err(|_| format!("{url}: {port} is not a port"))?,
        None => 80,
    };
    Ok(HttpTarget {
        host: host.to_string(),
        port,
        path: path.to_string(),
    })
}

/// Reads the status from the first bytes of a response; `None` when they are no HTTP status line.
pub fn parse_status_line(response: &[u8]) -> Option<u16> {
    let line = response.split(|byte| *byte == b'\n').next()?;
    let line = std::str::from_utf8(line).ok()?;
    let mut parts = line.trim_end().split(' ');
    if !parts.next()?.starts_with("HTTP/") {
        return None;
    }
    parts.next()?.parse().ok()
}

/// Asks `target` with one `GET` and returns the status it answers, each step bounded by `timeout`.
pub fn http_status(target: &HttpTarget, timeout: Duration) -> Result<u16, String> {
    let authority = target.authority();
    let (host, port) = (target.host(), target.port());
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("could not resolve {authority}: {e}"))?;
    let mut last = format!("{authority} resolved to no address");
    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => return exchange(stream, target, timeout),
            Err(e) => last = format!("could not connect to {address}: {e}"),
        }
    }
    Err(last)
}

fn exchange(mut stream: TcpStream, target: &HttpTarget, timeout: Duration) -> Result<u16, String> {
    let failed = |e: std::io::Error| format!("{e}");
    stream.set_read_timeout(Some(timeout)).map_err(failed)?;
    stream.set_write_timeout(Some(timeout)).map_err(failed)?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        target.path(),
        target.authority()
    );
    stream.write_all(request.as_bytes()).map_err(failed)?;

    let mut response = Vec::new();
    let mut chunk = [0u8; 512];
    while !response.contains(&b'\n') && response.len() < STATUS_LINE_MAX {
        let count = stream.read(&mut chunk).map_err(failed)?;
        if count == 0 {
            break;
        }
        response.extend_from_slice(&chunk[..count]);
    }
    parse_status_line(&response).ok_or_else(|| "the answer is no HTTP response".to_string())
}

/// Returns whether either captured stream contains the expected output.
///
/// `docker logs` keeps stdout and stderr apart rather than interleaving them, and a great
/// many services report readiness on stderr, in particular anything built on glog. Searching
/// stdout alone means those services can never match and the strategy can only time out.
pub fn streams_contain(stdout: &[u8], stderr: &[u8], expected_output: &str) -> bool {
    String::from_utf8_lossy(stdout).contains(expected_output)
        || String::from_utf8_lossy(stderr).contains(expected_output)
}
