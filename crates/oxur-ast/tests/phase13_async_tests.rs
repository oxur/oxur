use oxur_ast::integration::parse_rust_file;

/// Phase 13: Test basic async block
#[test]
fn test_async_block() {
    let code = r#"
        fn main() {
            let future = async {
                42
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse async block: {:?}", result.err());
}

/// Phase 13: Test async move block
#[test]
fn test_async_move_block() {
    let code = r#"
        fn main() {
            let data = vec![1, 2, 3];
            let future = async move {
                data.len()
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse async move block: {:?}", result.err());
}

/// Phase 13: Test await expression
#[test]
fn test_await_expression() {
    let code = r#"
        async fn example() -> Result<String, ()> {
            let result = fetch_data().await;
            Ok(result)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse await expression: {:?}", result.err());
}

/// Phase 13: Test async function
#[test]
fn test_async_function() {
    let code = r#"
        async fn fetch() -> Result<String, ()> {
            let response = client.get("url").await;
            Ok(response)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse async function: {:?}", result.err());
}

/// Phase 13: Test chained awaits
#[test]
fn test_chained_awaits() {
    let code = r#"
        async fn complex() -> Result<i32, ()> {
            fetch()
                .await
                .process()
                .await
                .finalize()
                .await
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse chained awaits: {:?}", result.err());
}

/// Phase 13: Test await with try operator
#[test]
fn test_await_with_try() {
    let code = r#"
        async fn example() -> Result<String, ()> {
            let data = fetch_data().await?;
            Ok(data)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse await with try: {:?}", result.err());
}

/// Phase 13: Test async method
#[test]
fn test_async_method() {
    let code = r#"
        impl MyStruct {
            async fn process(&self) -> Result<(), ()> {
                self.data.process().await
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse async method: {:?}", result.err());
}

/// Phase 13: Test nested async blocks
#[test]
fn test_nested_async_blocks() {
    let code = r#"
        fn main() {
            let outer = async {
                let inner = async {
                    42
                };
                inner.await
            };
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse nested async blocks: {:?}", result.err());
}

/// Phase 13: Test async block in match
#[test]
fn test_async_in_match() {
    let code = r#"
        async fn example(x: i32) -> i32 {
            match x {
                0 => async { 1 }.await,
                _ => async { 2 }.await,
            }
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse async in match: {:?}", result.err());
}

/// Phase 13: Integration test - real async pattern
#[test]
fn test_real_async_pattern() {
    let code = r#"
        async fn fetch_and_process() -> Result<Vec<i32>, ()> {
            let response = async {
                fetch_data().await
            }.await;

            let processed = response
                .into_iter()
                .map(|x| x * 2)
                .collect();

            Ok(processed)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse real async pattern: {:?}", result.err());
}

/// Phase 13: Integration test - async with error handling
#[test]
fn test_async_error_handling() {
    let code = r#"
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let response = reqwest::get("https://example.com")
                .await?
                .text()
                .await?;

            println!("{}", response);
            Ok(())
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse async error handling: {:?}", result.err());
}

/// Phase 13: Test async function with complex body
#[test]
fn test_async_complex_body() {
    let code = r#"
        async fn process_items(items: Vec<i32>) -> Result<Vec<i32>, ()> {
            let mut results = Vec::new();

            for item in items {
                let result = async move {
                    process_one(item).await
                }.await;

                results.push(result);
            }

            Ok(results)
        }
    "#;

    let result = parse_rust_file(code);
    assert!(result.is_ok(), "Failed to parse async complex body: {:?}", result.err());
}
