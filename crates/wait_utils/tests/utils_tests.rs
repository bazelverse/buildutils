/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The buildutils Authors and Contributors. All Rights Reserved.
 */

//! Tests for the pure helpers behind the wait strategies. These need neither a network nor Docker.

use wait_utils::utils_test::{parse_http_url, parse_status_line, streams_contain};

const EXPECTED: &str = "ready to accept requests";

#[test]
fn test_a_url_reads_as_its_host_port_and_path() {
    let target = parse_http_url("http://localhost:8080/health").expect("a URL");
    assert_eq!(target.host(), "localhost");
    assert_eq!(target.port(), 8080);
    assert_eq!(target.path(), "/health");
    assert_eq!(target.authority(), "localhost:8080");
}

#[test]
fn test_a_url_defaults_its_port_and_path() {
    let target = parse_http_url("http://example.org").expect("a URL");
    assert_eq!(target.port(), 80);
    assert_eq!(target.path(), "/");
}

#[test]
fn test_a_bracketed_ipv6_host_loses_its_brackets() {
    let target = parse_http_url("http://[::1]:8080/x").expect("a URL");
    assert_eq!(target.host(), "::1");
    assert_eq!(target.port(), 8080);
    assert_eq!(parse_http_url("http://[::1]/").expect("a URL").port(), 80);
    assert!(parse_http_url("http://[::1").is_err());
}

#[test]
fn test_a_url_that_cannot_be_probed_is_refused_by_name() {
    let https = parse_http_url("https://example.org/health").expect_err("https");
    assert!(https.contains("https is not supported"), "{https}");
    assert!(parse_http_url("ftp://example.org").is_err());
    assert!(parse_http_url("http://").is_err());
    assert!(parse_http_url("http://host:notaport/").is_err());
}

#[test]
fn test_a_status_line_gives_its_status() {
    assert_eq!(
        parse_status_line(b"HTTP/1.1 503 Service Unavailable\r\n"),
        Some(503)
    );
    assert_eq!(parse_status_line(b"HTTP/1.0 200\r\nHost: x\r\n"), Some(200));
    assert_eq!(parse_status_line(b"SSH-2.0-OpenSSH\r\n"), None);
    assert_eq!(parse_status_line(b"HTTP/1.1 abc\r\n"), None);
    assert_eq!(parse_status_line(b""), None);
    assert_eq!(parse_status_line(&[0xff, 0xfe]), None);
}

#[test]
fn test_streams_contain_matches_stdout() {
    let stdout = b"server started, ready to accept requests\n";
    assert!(streams_contain(stdout, b"", EXPECTED));
}

#[test]
fn test_streams_contain_matches_stderr() {
    // Services built on glog, such as dgraph, log their readiness to stderr only.
    let stderr = b"I0728 13:00:00.000000 1 server.go:42] ready to accept requests\n";
    assert!(streams_contain(b"", stderr, EXPECTED));
}

#[test]
fn test_streams_contain_matches_either_stream() {
    let stdout = b"ready to accept requests\n";
    let stderr = b"ready to accept requests\n";
    assert!(streams_contain(stdout, stderr, EXPECTED));
}

#[test]
fn test_streams_contain_without_a_match() {
    assert!(!streams_contain(
        b"starting up\n",
        b"binding port\n",
        EXPECTED
    ));
}

#[test]
fn test_streams_contain_of_empty_output() {
    assert!(!streams_contain(b"", b"", EXPECTED));
}

#[test]
fn test_streams_contain_of_invalid_utf8() {
    // Container output is arbitrary bytes and must not panic the search.
    let stderr = [0xff, 0xfe, b'r', b'e', b'a', b'd', b'y'];

    assert!(!streams_contain(b"", &stderr, EXPECTED));
    assert!(streams_contain(b"", &stderr, "ready"));
}
