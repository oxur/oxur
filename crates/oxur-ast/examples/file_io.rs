use oxur_ast::sexp::{print_sexp, write_sexp_file, Parser};
use std::path::PathBuf;

fn main() {
    println!("=== S-Expression File I/O Examples ===\n");

    // Get paths to example files
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let test_data = PathBuf::from(manifest_dir).join("test-data");

    // Example 1: Reading simple S-expressions from files
    println!("1. Reading Simple Examples");
    println!("   ------------------------");

    let simple_files = vec![
        "examples/simple/nil-value.sexp",
        "examples/simple/number.sexp",
        "examples/simple/symbol.sexp",
        "examples/simple/string.sexp",
    ];

    for file in simple_files {
        let path = test_data.join(file);
        match Parser::parse_file(&path) {
            Ok(sexp) => {
                println!("   ✓ {}: {}", file, print_sexp(&sexp));
            }
            Err(e) => {
                eprintln!("   ✗ Failed to parse {}: {}", file, e);
            }
        }
    }

    // Example 2: Reading intermediate complexity examples
    println!("\n2. Reading Intermediate Examples");
    println!("   ------------------------------");

    let intermediate_file = test_data.join("examples/intermediate/simple-fn.sexp");
    match Parser::parse_file(&intermediate_file) {
        Ok(sexp) => {
            println!("   ✓ Parsed simple-fn.sexp");
            println!(
                "   First 100 chars: {}...",
                print_sexp(&sexp).chars().take(100).collect::<String>()
            );
        }
        Err(e) => {
            eprintln!("   ✗ Failed to parse: {}", e);
        }
    }

    // Example 3: Reading complex nested structures
    println!("\n3. Reading Complex Examples");
    println!("   -------------------------");

    let complex_file = test_data.join("examples/complex/full-crate.sexp");
    match Parser::parse_file(&complex_file) {
        Ok(sexp) => {
            println!("   ✓ Parsed full-crate.sexp");
            let output = print_sexp(&sexp);
            println!("   Total size: {} characters", output.len());
            println!("   First line: {}", output.lines().next().unwrap_or(""));
        }
        Err(e) => {
            eprintln!("   ✗ Failed to parse: {}", e);
        }
    }

    // Example 4: Writing S-expressions to files
    println!("\n4. Writing S-expressions to Files");
    println!("   -------------------------------");

    // Read an existing S-expression
    let source_file = test_data.join("examples/simple/empty-list.sexp");
    match Parser::parse_file(&source_file) {
        Ok(sexp) => {
            // Write it to a temporary file
            let temp_dir = std::env::temp_dir();
            let output_file = temp_dir.join("oxur-ast-example-output.sexp");

            match write_sexp_file(&sexp, &output_file) {
                Ok(()) => {
                    println!("   ✓ Written to: {:?}", output_file);
                    println!("   Content: {}", print_sexp(&sexp));
                }
                Err(e) => {
                    eprintln!("   ✗ Failed to write: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("   ✗ Failed to read source: {}", e);
        }
    }

    // Example 5: Round-trip (read → write → read)
    println!("\n5. Round-Trip File I/O");
    println!("   --------------------");

    let original_file = test_data.join("examples/intermediate/macro-call.sexp");
    match Parser::parse_file(&original_file) {
        Ok(original) => {
            let temp_dir = std::env::temp_dir();
            let roundtrip_file = temp_dir.join("oxur-ast-roundtrip.sexp");

            // Write to file
            if let Err(e) = write_sexp_file(&original, &roundtrip_file) {
                eprintln!("   ✗ Failed to write: {}", e);
                return;
            }

            // Read back
            match Parser::parse_file(&roundtrip_file) {
                Ok(reparsed) => {
                    println!("   ✓ Round-trip successful!");
                    println!(
                        "   Original and reparsed structures match: {}",
                        print_sexp(&original) == print_sexp(&reparsed)
                    );

                    // Clean up
                    let _ = std::fs::remove_file(&roundtrip_file);
                }
                Err(e) => {
                    eprintln!("   ✗ Failed to re-parse: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("   ✗ Failed to read original: {}", e);
        }
    }

    // Example 6: Error handling
    println!("\n6. Error Handling");
    println!("   ---------------");

    let error_files = vec![
        ("unterminated-list.sexp", "UnterminatedList"),
        ("unexpected-close.sexp", "UnexpectedCloseParen"),
        ("invalid-escape.sexp", "Invalid escape sequence"),
    ];

    for (file, expected_error) in error_files {
        let path = test_data.join("error-cases").join(file);
        match Parser::parse_file(&path) {
            Ok(_) => {
                println!("   ✗ {}: Should have failed!", file);
            }
            Err(e) => {
                println!("   ✓ {}: Correctly rejected ({})", file, expected_error);
                println!("      Error: {}", e);
            }
        }
    }

    println!("\n=== All examples completed! ===");
}
