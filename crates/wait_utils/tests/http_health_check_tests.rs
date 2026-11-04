/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use wait_utils::wait_until_http_health_check;

/// Serves a fixed HTTP status line on a free loopback port and returns that port.
///
/// The serving thread is left running for the lifetime of the test binary, which keeps the
/// helper simple and costs nothing once the process exits.
fn spawn_http_server(status_line: &'static str, body: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a test port");
    let port = listener
        .local_addr()
        .expect("Failed to read the test port")
        .port();

    thread::spawn(move || {
        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );

        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };

            // The request is read and discarded; only the response matters here.
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    port
}

/// A health endpoint that reports it is not ready must not end the wait.
///
/// Dgraph answers /health with 503 and "Please retry again, server is not ready to accept
/// requests" while it is still starting, which is a health endpoint doing its job, so a
/// response is not enough: its status must say ready.
#[test]
fn test_http_health_check_rejects_a_503() {
    let port = spawn_http_server(
        "503 Service Unavailable",
        "Please retry again, server is not ready to accept requests",
    );
    let health_url = format!("http://127.0.0.1:{port}/health");

    let res = wait_until_http_health_check(false, &health_url, &2);

    assert!(res.is_err(), "A 503 must not be accepted as ready");
    assert!(
        res.unwrap_err().to_string().contains("Timeout"),
        "The wait must run to its timeout rather than succeed"
    );
}

/// Any other error status must be rejected as well.
#[test]
fn test_http_health_check_rejects_a_404() {
    let port = spawn_http_server("404 Not Found", "not found");
    let health_url = format!("http://127.0.0.1:{port}/health");

    let res = wait_until_http_health_check(false, &health_url, &2);

    assert!(res.is_err(), "A 404 must not be accepted as ready");
}

/// A success status ends the wait, so the check above is not simply rejecting everything.
#[test]
fn test_http_health_check_accepts_a_200() {
    let port = spawn_http_server("200 OK", "OK");
    let health_url = format!("http://127.0.0.1:{port}/health");

    let res = wait_until_http_health_check(false, &health_url, &10);

    assert!(res.is_ok(), "A 200 must be accepted as ready: {res:?}");
}

/// Nothing listening is a timeout, not a success.
#[test]
fn test_http_health_check_times_out_when_nothing_listens() {
    // Bound and immediately dropped, so the port is known to be free.
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a test port");
        listener.local_addr().expect("Failed to read port").port()
    };
    let health_url = format!("http://127.0.0.1:{port}/health");

    let res = wait_until_http_health_check(false, &health_url, &2);

    assert!(res.is_err(), "An unreachable endpoint must not be ready");
}

/// A URL that can never be probed fails at once, rather than after its whole timeout.
#[test]
fn test_http_health_check_refuses_https_at_once() {
    let started = std::time::Instant::now();
    let refused = wait_until_http_health_check(false, "https://localhost:1/health", &30);
    assert!(refused.is_err());
    assert!(started.elapsed().as_secs() < 5, "it waited out the timeout");
}
