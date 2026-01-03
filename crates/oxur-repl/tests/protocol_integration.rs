// Integration tests for the complete REPL protocol stack
//
// Tests the full end-to-end flow: Protocol messages -> Postcard codec -> TCP transport

use oxur_repl::protocol::{
    ErrorInfo, ErrorKind, Operation, OperationResult, ReplMode, Request, Response, SourceLocation,
    Status,
};
use oxur_repl::transport::{
    SplitTransport, TcpTransport, TcpTransportListener, Transport, TransportReader,
    TransportWriter,
};

/// Test basic request/response cycle over TCP
#[tokio::test]
async fn test_basic_request_response() {
    let listener = TcpTransportListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind listener");
    let addr = listener.local_addr().expect("Failed to get address");

    let request = Request {
        id: 1,
        session_id: "test-session-1".to_string(),
        operation: Operation::Eval {
            code: "(+ 1 2)".to_string(),
            mode: ReplMode::Lisp,
        },
    };

    let response = Response {
        request_id: 1,
        session_id: "test-session-1".to_string(),
        result: OperationResult::Success {
            status: Status {
                tier: 1,
                cached: false,
                duration_ms: 5,
            },
            value: Some("3".to_string()),
            stdout: None,
            stderr: None,
        },
    };

    let request_clone = request.clone();
    let response_clone = response.clone();

    // Client task
    let client_task = tokio::spawn(async move {
        let mut client = TcpTransport::connect(addr.to_string())
            .await
            .expect("Client failed to connect");

        // Send request
        client
            .send_request(&request_clone)
            .await
            .expect("Failed to send request");

        // Receive response
        let recv_response = client
            .recv_response()
            .await
            .expect("Failed to receive response");

        assert_eq!(recv_response.request_id, 1);
        assert_eq!(recv_response.session_id, "test-session-1");

        if let OperationResult::Success { value, .. } = recv_response.result {
            assert_eq!(value, Some("3".to_string()));
        } else {
            panic!("Expected Success result");
        }
    });

    // Server task
    let server_task = tokio::spawn(async move {
        let mut server = listener.accept().await.expect("Failed to accept");

        // Receive request
        let recv_request = server
            .recv_request()
            .await
            .expect("Failed to receive request");

        assert_eq!(recv_request.id, 1);
        assert_eq!(recv_request.session_id, "test-session-1");

        // Send response
        server
            .send_response(&response_clone)
            .await
            .expect("Failed to send response");
    });

    tokio::try_join!(client_task, server_task).expect("Tasks failed");
}

/// Test multiple sequential requests over the same connection
#[tokio::test]
async fn test_multiple_requests() {
    let listener = TcpTransportListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get address");

    // Client task - send 3 requests
    let client_task = tokio::spawn(async move {
        let mut client = TcpTransport::connect(addr.to_string())
            .await
            .expect("Failed to connect");

        for i in 1..=3 {
            let request = Request {
                id: i,
                session_id: "multi-test".to_string(),
                operation: Operation::Eval {
                    code: format!("(+ {} {})", i, i),
                    mode: ReplMode::Lisp,
                },
            };

            client
                .send_request(&request)
                .await
                .expect("Failed to send request");

            let response = client
                .recv_response()
                .await
                .expect("Failed to receive response");

            assert_eq!(response.request_id, i);
        }
    });

    // Server task - respond to 3 requests
    let server_task = tokio::spawn(async move {
        let mut server = listener.accept().await.expect("Failed to accept");

        for i in 1..=3 {
            let request = server
                .recv_request()
                .await
                .expect("Failed to receive request");

            assert_eq!(request.id, i);

            let response = Response {
                request_id: i,
                session_id: "multi-test".to_string(),
                result: OperationResult::Success {
                    status: Status {
                        tier: 1,
                        cached: false,
                        duration_ms: 5,
                    },
                    value: Some((i * 2).to_string()),
                    stdout: None,
                    stderr: None,
                },
            };

            server
                .send_response(&response)
                .await
                .expect("Failed to send response");
        }
    });

    tokio::try_join!(client_task, server_task).expect("Tasks failed");
}

/// Test error response propagation
#[tokio::test]
async fn test_error_response() {
    let listener = TcpTransportListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get address");

    let error_response = Response {
        request_id: 1,
        session_id: "error-test".to_string(),
        result: OperationResult::Error {
            error: ErrorInfo {
                kind: ErrorKind::RuntimeError,
                message: "Division by zero".to_string(),
                location: Some(SourceLocation {
                    offset: 10,
                    line: 2,
                    column: 5,
                }),
                details: Some("Stack trace: ...".to_string()),
            },
            stdout: Some("Processing...".to_string()),
            stderr: Some("Error: division by zero".to_string()),
        },
    };

    let error_clone = error_response.clone();

    let client_task = tokio::spawn(async move {
        let mut client = TcpTransport::connect(addr.to_string())
            .await
            .expect("Failed to connect");

        let request = Request {
            id: 1,
            session_id: "error-test".to_string(),
            operation: Operation::Eval {
                code: "(/ 1 0)".to_string(),
                mode: ReplMode::Lisp,
            },
        };

        client
            .send_request(&request)
            .await
            .expect("Failed to send request");

        let response = client
            .recv_response()
            .await
            .expect("Failed to receive response");

        if let OperationResult::Error {
            error,
            stdout,
            stderr,
        } = response.result
        {
            assert_eq!(error.kind, ErrorKind::RuntimeError);
            assert_eq!(error.message, "Division by zero");
            assert!(error.location.is_some());
            assert_eq!(stdout, Some("Processing...".to_string()));
            assert_eq!(stderr, Some("Error: division by zero".to_string()));
        } else {
            panic!("Expected Error result");
        }
    });

    let server_task = tokio::spawn(async move {
        let mut server = listener.accept().await.expect("Failed to accept");
        let _request = server.recv_request().await.expect("Failed to receive");
        server
            .send_response(&error_clone)
            .await
            .expect("Failed to send error response");
    });

    tokio::try_join!(client_task, server_task).expect("Tasks failed");
}

/// Test all operation types
#[tokio::test]
async fn test_all_operations() {
    let listener = TcpTransportListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get address");

    let operations = vec![
        Operation::Eval {
            code: "(+ 1 2)".to_string(),
            mode: ReplMode::Lisp,
        },
        Operation::LoadFile {
            path: "test.ox".to_string(),
            mode: ReplMode::Sexpr,
        },
        Operation::Clone {
            source_session_id: "source-session".to_string(),
        },
        Operation::Interrupt,
        Operation::Close,
        Operation::LsSessions,
        Operation::Describe {
            symbol: "map".to_string(),
        },
        Operation::History { limit: Some(10) },
        Operation::ClearOutput,
    ];

    let ops_clone = operations.clone();

    let client_task = tokio::spawn(async move {
        let mut client = TcpTransport::connect(addr.to_string())
            .await
            .expect("Failed to connect");

        for (i, op) in ops_clone.into_iter().enumerate() {
            let request = Request {
                id: i as u64,
                session_id: "ops-test".to_string(),
                operation: op,
            };

            client
                .send_request(&request)
                .await
                .expect("Failed to send request");

            let response = client
                .recv_response()
                .await
                .expect("Failed to receive response");

            assert_eq!(response.request_id, i as u64);
        }
    });

    let server_task = tokio::spawn(async move {
        let mut server = listener.accept().await.expect("Failed to accept");

        for i in 0..operations.len() {
            let request = server
                .recv_request()
                .await
                .expect("Failed to receive request");

            assert_eq!(request.id, i as u64);

            let response = Response {
                request_id: i as u64,
                session_id: "ops-test".to_string(),
                result: OperationResult::Success {
                    status: Status {
                        tier: 1,
                        cached: false,
                        duration_ms: 1,
                    },
                    value: Some("ok".to_string()),
                    stdout: None,
                    stderr: None,
                },
            };

            server
                .send_response(&response)
                .await
                .expect("Failed to send response");
        }
    });

    tokio::try_join!(client_task, server_task).expect("Tasks failed");
}

/// Test concurrent operations using split transport
#[tokio::test]
async fn test_concurrent_split_transport() {
    let listener = TcpTransportListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get address");

    // Client with split transport
    let client_task = tokio::spawn(async move {
        let client = TcpTransport::connect(addr.to_string())
            .await
            .expect("Failed to connect");

        let (mut reader, mut writer) = client.split();

        // Spawn sender task
        let send_task = tokio::spawn(async move {
            for i in 1..=5 {
                let request = Request {
                    id: i,
                    session_id: "concurrent".to_string(),
                    operation: Operation::Eval {
                        code: format!("request-{}", i),
                        mode: ReplMode::Lisp,
                    },
                };

                writer
                    .send_request(&request)
                    .await
                    .expect("Failed to send");
                writer.flush().await.expect("Failed to flush");
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        // Receive responses in main task
        let recv_task = tokio::spawn(async move {
            for i in 1..=5 {
                let response = reader
                    .recv_response()
                    .await
                    .expect("Failed to receive response");
                assert_eq!(response.request_id, i);
            }
        });

        tokio::try_join!(send_task, recv_task).expect("Split tasks failed");
    });

    // Server with split transport
    let server_task = tokio::spawn(async move {
        let server = listener.accept().await.expect("Failed to accept");
        let (mut reader, mut writer) = server.split();

        // Spawn receiver task
        let recv_task = tokio::spawn(async move {
            for i in 1..=5 {
                let request = reader
                    .recv_request()
                    .await
                    .expect("Failed to receive request");
                assert_eq!(request.id, i);
            }
        });

        // Send responses
        let send_task = tokio::spawn(async move {
            for i in 1..=5 {
                let response = Response {
                    request_id: i,
                    session_id: "concurrent".to_string(),
                    result: OperationResult::Success {
                        status: Status {
                            tier: 1,
                            cached: false,
                            duration_ms: 1,
                        },
                        value: Some(format!("response-{}", i)),
                        stdout: None,
                        stderr: None,
                    },
                };

                writer
                    .send_response(&response)
                    .await
                    .expect("Failed to send response");
                writer.flush().await.expect("Failed to flush");
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
        });

        tokio::try_join!(recv_task, send_task).expect("Server split tasks failed");
    });

    tokio::try_join!(client_task, server_task).expect("Tasks failed");
}

/// Test large messages (close to size limit)
#[tokio::test]
async fn test_large_message() {
    let listener = TcpTransportListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get address");

    // Create a large code string (1MB)
    let large_code = "x".repeat(1024 * 1024);

    let client_task = tokio::spawn(async move {
        let mut client = TcpTransport::connect(addr.to_string())
            .await
            .expect("Failed to connect");

        let request = Request {
            id: 1,
            session_id: "large-msg".to_string(),
            operation: Operation::Eval {
                code: large_code,
                mode: ReplMode::Lisp,
            },
        };

        client
            .send_request(&request)
            .await
            .expect("Failed to send large request");

        let response = client
            .recv_response()
            .await
            .expect("Failed to receive response");

        assert_eq!(response.request_id, 1);
    });

    let server_task = tokio::spawn(async move {
        let mut server = listener.accept().await.expect("Failed to accept");

        let request = server
            .recv_request()
            .await
            .expect("Failed to receive large request");

        assert_eq!(request.id, 1);

        // Verify we received the large code
        if let Operation::Eval { code, .. } = request.operation {
            assert_eq!(code.len(), 1024 * 1024);
        }

        let response = Response {
            request_id: 1,
            session_id: "large-msg".to_string(),
            result: OperationResult::Success {
                status: Status {
                    tier: 2,
                    cached: false,
                    duration_ms: 100,
                },
                value: Some("processed".to_string()),
                stdout: None,
                stderr: None,
            },
        };

        server
            .send_response(&response)
            .await
            .expect("Failed to send response");
    });

    tokio::try_join!(client_task, server_task).expect("Tasks failed");
}

/// Test connection cleanup and graceful shutdown
#[tokio::test]
async fn test_graceful_shutdown() {
    let listener = TcpTransportListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get address");

    let client_task = tokio::spawn(async move {
        let mut client = TcpTransport::connect(addr.to_string())
            .await
            .expect("Failed to connect");

        let request = Request {
            id: 1,
            session_id: "shutdown-test".to_string(),
            operation: Operation::Close,
        };

        client
            .send_request(&request)
            .await
            .expect("Failed to send");

        let _response = client.recv_response().await.expect("Failed to receive");

        // Close the connection
        client.close().await.expect("Failed to close");
    });

    let server_task = tokio::spawn(async move {
        let mut server = listener.accept().await.expect("Failed to accept");

        let _request = server.recv_request().await.expect("Failed to receive");

        let response = Response {
            request_id: 1,
            session_id: "shutdown-test".to_string(),
            result: OperationResult::Success {
                status: Status {
                    tier: 1,
                    cached: false,
                    duration_ms: 0,
                },
                value: None,
                stdout: None,
                stderr: None,
            },
        };

        server
            .send_response(&response)
            .await
            .expect("Failed to send");

        // Server closes connection
        server.close().await.expect("Failed to close");
    });

    tokio::try_join!(client_task, server_task).expect("Tasks failed");
}
