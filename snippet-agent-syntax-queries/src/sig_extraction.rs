use std::result::Result;
use tree_sitter::{Node, Parser, Tree};

pub struct RustParser<'a> {
    /// Borrowed source (lifetime tied to the caller's source buffer).
    pub source: &'a str,
    pub tree: Tree,
    /// The piece of text to look for inside the node we want to delete.
    pub target_node_text: String,
}

///delete_till_start
impl<'a> RustParser<'a> {
    pub fn delete_till_start(&self, target: &str) -> Option<String> {
        let root = self.tree.root_node();

        // Find the node of the requested kind.
        let node = Self::find_node_of_kind(root, target)?;

        // We want to remove from the beginning of the source up to node.end_byte()
        let end = node.end_byte();

        // `after` is the remainder of the source after the node's end.
        let after = self.source.get(end..).unwrap_or("");

        // Build result and trim leading whitespace (user requested trimming from start).
        let mut result = String::with_capacity(self.source.len().saturating_sub(end));
        result.push_str(after);

        let trimmed_result = result.trim_start().to_string();

        Some(trimmed_result)
    }
}

///delete till end
impl<'a> RustParser<'a> {
    pub fn delete_node_till_end(&self) -> Option<String> {
        let root = self.tree.root_node();

        // Use the existing field name (target_node_text) as the kind string.
        let node = Self::find_node_of_kind(root, &self.target_node_text)?;

        let start = node.start_byte();
        let end = node.end_byte();

        // Be safe slicing the &str: use get(..) which returns Option<&str>.
        let before = self.source.get(..start).unwrap_or("");
        let after = self.source.get(end..).unwrap_or("");

        let mut result =
            String::with_capacity(self.source.len().saturating_sub(end.saturating_sub(start)));
        result.push_str(before);
        result.push_str(after);

        let trimmed_result = result.trim_end().to_string();

        Some(trimmed_result)
    }
}

///save type identifier
impl<'a> RustParser<'a> {
    /// Populate `out` with the text of every node whose kind equals
    /// `self.target_node_text`. Traverses the tree in source (left-to-right)
    /// order and pushes each match into `out`.
    pub fn save_type_identifiers(&self, out: &mut Vec<String>) {
        let root = self.tree.root_node();
        let kind = &self.target_node_text;

        // Iterative DFS that preserves left-to-right order by pushing children in reverse.
        let mut stack = vec![root];

        while let Some(node) = stack.pop() {
            if node.kind() == kind {
                let start = node.start_byte();
                let end = node.end_byte();
                // use get(..).unwrap_or("") to avoid panics on invalid slicing
                let text = self.source.get(start..end).unwrap_or("").to_owned();
                out.push(text);
            }

            let child_count = node.child_count();
            // push children in reverse so the left-most child is processed first
            for i in (0..child_count).rev() {
                if let Some(child) = node.child(i) {
                    stack.push(child);
                }
            }
        }
    }
}

///select last
impl<'a> RustParser<'a> {
    /// Return the source text of the *last* node whose kind equals
    /// `self.target_node_text`. If no matching node is found, returns an
    /// empty `String`.
    pub fn select_last(&self) -> String {
        let root = self.tree.root_node();
        let kind = &self.target_node_text;

        let mut last_match: Option<tree_sitter::Node> = None;
        let mut stack = vec![root];

        while let Some(node) = stack.pop() {
            if node.kind() == kind {
                match last_match {
                    None => last_match = Some(node),
                    Some(prev) if node.start_byte() > prev.start_byte() => last_match = Some(node),
                    _ => {}
                }
            }

            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    stack.push(child);
                }
            }
        }

        if let Some(n) = last_match {
            let start = n.start_byte();
            let end = n.end_byte();
            // get(..) returns Option<&str> to avoid panics; fallback to "" if invalid
            self.source.get(start..end).unwrap_or("").to_owned()
        } else {
            String::new()
        }
    }
}

///delete all nodes
impl<'a> RustParser<'a> {
    /// Remove *all* nodes whose kind matches `self.target_node_text`.
    /// Returns `Some(new_source)` with those nodes removed, or `None` if none found.
    pub fn delete_all_nodes(&self) -> String {
        let root = self.tree.root_node();

        // Collect all matching node ranges (start_byte, end_byte).
        let mut ranges: Vec<(usize, usize)> = Vec::new();

        // Recursive helper to walk the tree and collect ranges for nodes whose kind() matches `kind`.
        fn collect_ranges<'n>(node: Node<'n>, kind: &str, out: &mut Vec<(usize, usize)>) {
            if node.kind() == kind {
                out.push((node.start_byte(), node.end_byte()));
            }

            let child_count = node.child_count();
            for i in 0..child_count {
                if let Some(child) = node.child(i) {
                    collect_ranges(child, kind, out);
                }
            }
        }

        collect_ranges(root, &self.target_node_text, &mut ranges);

        if ranges.is_empty() {
            return self.source.to_string();
        }

        // Sort by start byte
        ranges.sort_by_key(|r| r.0);

        // Merge overlapping or adjacent ranges
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (start, end) in ranges {
            if let Some(last) = merged.last_mut() {
                if start <= last.1 {
                    // overlap or adjacent -> extend end if needed
                    if end > last.1 {
                        last.1 = end;
                    }
                    continue;
                }
            }
            merged.push((start, end));
        }

        // Build result by keeping slices outside merged ranges
        let total_removed: usize = merged.iter().map(|(s, e)| e.saturating_sub(*s)).sum();
        let mut result = String::with_capacity(self.source.len().saturating_sub(total_removed));

        let mut prev_end = 0usize;
        for (start, end) in &merged {
            // append the part before this range
            result.push_str(self.source.get(prev_end..*start).unwrap_or(""));
            prev_end = *end;
        }
        // append the tail
        result.push_str(self.source.get(prev_end..).unwrap_or(""));

        result
    }
}

///find node
impl<'a> RustParser<'a> {
    /// Recursive helper to find the first node whose kind equals `target_kind`.
    fn find_node_of_kind<'b>(node: Node<'b>, target_kind: &str) -> Option<Node<'b>> {
        if node.kind() == target_kind {
            return Some(node);
        }

        let mut walk = node.walk();
        for child in node.children(&mut walk) {
            if let Some(found) = Self::find_node_of_kind(child, target_kind) {
                return Some(found);
            }
        }

        None
    }
}

///new
impl<'a> RustParser<'a> {
    /// Creates a new parser and remembers the target text to look for when deleting.
    pub fn new(source: &'a str, target_node_text: &str) -> Result<Self, String> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::language())
            .map_err(|e| format!("failed to set language: {:?}", e))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| "failed to parse source".to_string())?;

        Ok(RustParser {
            source,
            tree,
            target_node_text: target_node_text.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_till_start_type_parameters() {
        let source = r#"impl<T, const N: usize> Array<T, N> where T: Clone + std::fmt::Debug + serde::de::Deserialize<'de>"#;
        // create parser (constructor usage follows your earlier test pattern)
        let parser =
            RustParser::new(source, "type_parameters").expect("failed to create RustParser");

        // Method takes the target node kind as a parameter
        let result = parser
            .delete_till_start("type_parameters")
            .expect("expected delete_till_start to find and remove the prefix");

        let expected = "Array<T, N> where T: Clone + std::fmt::Debug + serde::de::Deserialize<'de>";
        assert_eq!(result, expected);
    }

    #[test]
    fn delete_block() {
        // source must live long enough for the parser (string literal is fine)
        let source =
            "pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}";
        let target = "block";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        let result = parser
            .delete_node_till_end()
            .expect("expected delete_node_till_end to find and remove the block");

        let expected =
            "pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self";
        assert_eq!(result, expected);
    }

    #[test]
    fn delete_declaration_list() {
        // source must live long enough for the parser (string literal is fine)
        let source = r#"impl TraitToImpl {
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}}"#;
        let target = "declaration_list";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        let result = parser
            .delete_node_till_end()
            .expect("expected delete_node_till_end to find and remove the block");

        let expected = "impl TraitToImpl";
        assert_eq!(result, expected);
    }

    #[test]
    fn delete_type_arguments() {
        // source must live long enough for the parser (string literal is fine)
        let source = r#"impl<T> From<T> for Wrapper<T> {}"#;
        let target = "type_arguments";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        // delete_all_nodes currently returns a String, so use it directly
        let result = parser.delete_all_nodes();

        let expected = "impl<T> From for Wrapper {}";
        assert_eq!(result, expected);
    }

    #[test]
    fn delete_type_parameters() {
        // source must live long enough for the parser (string literal is fine)
        let source = r#"impl<T> From<T> for Wrapper<T> {}"#;
        let target = "type_parameters";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        // delete_all_nodes currently returns a String, so use it directly
        let result = parser.delete_all_nodes();

        let expected = "impl From<T> for Wrapper<T> {}";
        assert_eq!(result, expected);
    }

    #[test]
    fn select_last_type_identifier() {
        // source must live long enough for the parser (string literal is fine)
        let source = r#"impl From for Wrapper {}"#;
        let target = "type_identifier";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        let result = parser.select_last();

        let expected = "Wrapper";
        assert_eq!(result, expected);
    }

    #[test]
    fn collects_fn_type_identifiers() {
        // simple, predictable source with three type identifiers in left-to-right order
        let source = r#"fn f() -> (A, B, C) {}"#;
        let target = "type_identifier";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        let mut types: Vec<String> = Vec::new();
        parser.save_type_identifiers(&mut types);

        let expected = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert_eq!(types, expected);
    }

    #[test]
    fn collects_impl_type_identifiers() {
        // simple, predictable source with three type identifiers in left-to-right order
        let source = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug,
"#;
        let target = "type_identifier";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        let mut types: Vec<String> = Vec::new();
        parser.save_type_identifiers(&mut types);

        let expected = vec![
            "T".to_string(),
            "Clone".to_string(),
            "Array".to_string(),
            "T".to_string(),
            "N".to_string(),
            "T".to_string(),
            "Debug".to_string(),
        ];
        assert_eq!(types, expected);
    }

    #[test]
    fn collects_trait_bounds() {
        // simple, predictable source with three type identifiers in left-to-right order
        let source = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug,
"#;
        let target = "trait_bounds";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        let mut types: Vec<String> = Vec::new();
        parser.save_type_identifiers(&mut types);

        let expected = vec![": Clone".to_string(), ": std::fmt::Debug".to_string()];
        assert_eq!(types, expected);
    }

    #[test]
    fn collects_const_parameter() {
        // simple, predictable source with three type identifiers in left-to-right order
        let source = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug,
"#;
        let target = "const_parameter";

        let parser = RustParser::new(source, target).expect("failed to create RustParser");
        let mut types: Vec<String> = Vec::new();
        parser.save_type_identifiers(&mut types);

        let expected = vec!["const N: usize".to_string()];
        assert_eq!(types, expected);
    }
}
