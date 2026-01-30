#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_run_ast_grep_rule_with_empty_method() {
        // Create a temporary directory for the test Rust file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        // Create sample Rust code with an impl block
        let rust_code = r#"
struct MyStruct;

impl MyStruct {
    pub fn new() -> Self {
        Self
    }

    pub fn existing_method(&self) -> bool {
        true
    }
}

fn some_standalone_function() {
    println!("Not in an impl");
}
"#;

        // Write the Rust code to a file in the temp directory
        let rust_file_path = temp_dir.path().join("test.rs");
        fs::write(&rust_file_path, rust_code).expect("Failed to write test Rust file");

        // Run the function with empty method string
        let result = run_ast_grep_rule("", temp_dir.path().to_str().unwrap());

        // Check that the function succeeded
        let output = result.expect("run_ast_grep_rule should succeed");

        // Read the expected output from method_impl.json
        // Assuming the test file is in the same directory as this test
        let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
        let expected_path = test_dir.join("method_impl.json");

        // If the expected file doesn't exist, create it with the current output
        // (useful for initial test setup)
        if !expected_path.exists() {
            fs::write(&expected_path, &output).expect("Failed to write expected output");
        }

        let expected_output = fs::read_to_string(&expected_path)
            .expect("Failed to read method_impl.json");

        // Compare byte by byte
        assert_eq!(
            output, expected_output,
            "Output doesn't match expected content in method_impl.json"
        );
    }
}
