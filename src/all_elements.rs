use crate::ast_grep::ast_grep_everything_selection::extract_selections_from_ast_grep_json;
use crate::json_selection::unprocessed_elements::*;

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use walkdir::WalkDir;

///print_all
impl AllUnprocessedElements {
    /// Print a human-friendly, tabbed summary of the parsed syn elements.
    /// - File is printed at the start of each line (filename only).
    /// - Other fields are printed after that, separated by tabs, as `key: 'value'`.
    /// - Body-like fields are abbreviated and end with `..` when truncated.
    pub fn print_all(&self) {
        const MAX_BODY_PREVIEW: usize = 100;

        fn abbrev_body(s: &str, max: usize) -> String {
            let s = s.replace('\r', " ").replace('\n', " ");
            let s = s.trim();
            if s.chars().count() > max {
                s.chars().take(max).collect::<String>() + ".."
            } else {
                s.to_string()
            }
        }

        fn basename(path: &str) -> &str {
            path.rsplit('/').next().unwrap_or(path)
        }

        // Color functions now return plain text (no ANSI codes)
        fn green(s: &str) -> String {
            s.to_string()
        }

        fn yellow(s: &str) -> String {
            s.to_string()
        }

        fn purple(s: &str) -> String {
            s.to_string()
        }

        // print a group title (previously green)
        fn print_title(title: &str) {
            println!("{}", title);
        }

        // print a single indented field line
        fn print_field(indent: usize, key: &str, value: &str, is_name: bool, is_preview: bool) {
            let indent_str = "\t".repeat(indent);
            let displayed = if is_name {
                yellow(value)
            } else if is_preview {
                purple(value)
            } else {
                value.to_string()
            };
            println!("{}{}: '{}'", indent_str, key, displayed);
        }

        // Attributes
        if !self.unprocessed_attributes.is_empty() {
            print_title("Attributes:");
            for a in &self.unprocessed_attributes {
                let file = basename(&a.file);
                println!("\t{}", file);
                print_field(1, "attribute", &a.attribute_body.text, false, false);
                println!();
            }
        }

        // Test modules
        if !self.unprocessed_tests_mods.is_empty() {
            print_title("Test modules:");
            for t in &self.unprocessed_tests_mods {
                let file = basename(&t.file);
                println!("\t{}", green(file));
                print_field(
                    1,
                    "body",
                    &abbrev_body(&t.tests_mod_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Functions
        if !self.unprocessed_functions.is_empty() {
            print_title("Functions:");
            for f in &self.unprocessed_functions {
                let file = basename(&f.file);
                println!("\t{}", green(file));

                print_field(1, "name", &f.function_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&f.function_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Methods (impl methods)
        if !self.unprocessed_methods.is_empty() {
            print_title("Methods:");
            for m in &self.unprocessed_methods {
                let file = basename(&m.file);
                println!("\t{}", green(file));
                print_field(1, "name", &m.method_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&m.method_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                print_field(
                    1,
                    "impl",
                    &abbrev_body(&m.impl_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Impl blocks
        if !self.unprocessed_impls.is_empty() {
            print_title("Impl blocks:");
            for i in &self.unprocessed_impls {
                let file = basename(&i.file);
                println!("\t{}", green(file));
                print_field(
                    1,
                    "body",
                    &abbrev_body(&i.impl_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Structs
        if !self.unprocessed_structs.is_empty() {
            print_title("Structs:");
            for s in &self.unprocessed_structs {
                let file = basename(&s.file);
                println!("\t{}", green(file));
                print_field(1, "name", &s.struct_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&s.struct_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Traits
        if !self.unprocessed_traits.is_empty() {
            print_title("Traits:");
            for t in &self.unprocessed_traits {
                let file = basename(&t.file);
                println!("\t{}", green(file));
                print_field(1, "name", &t.trait_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&t.trait_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Trait method signatures
        if !self.unprocessed_trait_method_sigs.is_empty() {
            print_title("Trait method signatures:");
            for s in &self.unprocessed_trait_method_sigs {
                let file = basename(&s.file);
                println!("\t{}", green(file));
                print_field(
                    1,
                    "signature",
                    &abbrev_body(&s.trait_method_signature.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                print_field(1, "sig_name", &s.method_signature_name.text, true, false);
                print_field(
                    1,
                    "enclosing_trait_text",
                    &abbrev_body(&s.trait_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                print_field(1, "enclosing_trait", &s.trait_name.text, true, false);
                println!();
            }
        }

        // Trait method definitions (with bodies in traits)
        if !self.unprocessed_trait_method_defs.is_empty() {
            print_title("Trait method definitions:");
            for d in &self.unprocessed_trait_method_defs {
                let file = basename(&d.file);
                println!("\t{}", green(file));
                print_field(1, "method", &d.method_name.text, true, false);
                print_field(1, "trait", &d.trait_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&d.trait_method_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                print_field(
                    1,
                    "trait_body",
                    &abbrev_body(&d.trait_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Type aliases
        if !self.unprocessed_type_aliases.is_empty() {
            print_title("Type aliases:");
            for ta in &self.unprocessed_type_aliases {
                let file = basename(&ta.file);
                println!("\t{}", green(file));
                print_field(1, "name", &ta.type_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&ta.type_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Enums
        if !self.unprocessed_enums.is_empty() {
            print_title("Enums:");
            for e in &self.unprocessed_enums {
                let file = basename(&e.file);
                println!("\t{}", green(file));
                print_field(1, "name", &e.enum_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&e.enum_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }

        // Unions
        if !self.unprocessed_unions.is_empty() {
            print_title("Unions:");
            for u in &self.unprocessed_unions {
                let file = basename(&u.file);
                println!("\t{}", green(file));
                print_field(1, "name", &u.union_name.text, true, false);
                print_field(
                    1,
                    "body",
                    &abbrev_body(&u.union_body.text, MAX_BODY_PREVIEW),
                    false,
                    true,
                );
                println!();
            }
        }
    }
}

///from_json
impl AllUnprocessedElements {
    /// Parse the `ast-grep` JSON (the same input for `extract_selections_from_ast_grep_json`)
    /// and return an `AllUnprocessedElements` with each selection converted into its `Unprocessed*` counterpart.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let (
unprocessed_attributes,
unprocessed_tests_mods,
unprocessed_functions,
unprocessed_methods,
unprocessed_impls,
unprocessed_structs,
unprocessed_traits,
unprocessed_trait_method_sigs,
unprocessed_trait_method_defs,
unprocessed_type_aliases,
unprocessed_enums,
unprocessed_unions,
        ) = extract_selections_from_ast_grep_json(json)?;

        Ok(AllUnprocessedElements {
unprocessed_attributes: unprocessed_attributes
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedAttributes>(),
unprocessed_tests_mods: unprocessed_tests_mods
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedTestsMods>(),
unprocessed_functions: unprocessed_functions
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedFunctions>(),
unprocessed_methods: unprocessed_methods
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedMethods>(),
unprocessed_impls: unprocessed_impls
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedImpls>(),
unprocessed_structs: unprocessed_structs
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedStructs>(),
unprocessed_traits: unprocessed_traits
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedTraits>(),
unprocessed_trait_method_sigs: unprocessed_trait_method_sigs
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedTraitMethodSigs>(),
unprocessed_trait_method_defs: unprocessed_trait_method_defs
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedTraitMethodDefs>(),
unprocessed_type_aliases: unprocessed_type_aliases
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedTypeAliases>(),
unprocessed_enums: unprocessed_enums
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedEnums>(),
unprocessed_unions: unprocessed_unions
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect::<UnprocessedUnions>(),
        })
    }
}


/// compute length of a ByteRange (end >= start expected)
fn range_len(r: &ByteRange) -> u64 {
    if r.end >= r.start { r.end - r.start } else { 0 }
}

/// Trait to return the largest byte range that represents an "unprocessed element"
pub trait LargestByteRange {
    /// Return the largest byte range (by length) belonging to this element, if any.
    fn largest_byte_range(&self) -> Option<ByteRange>;
}

/// Helper to pick the larger of two ByteRanges
fn pick_larger(a: Option<ByteRange>, b: Option<ByteRange>) -> Option<ByteRange> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (Some(x), Some(y)) => {
            if range_len(&x) >= range_len(&y) { Some(x) } else { Some(y) }
        }
    }
}

/* ---------- Implement LargestByteRange for each Unprocessed type ---------- */
/* For types with multiple SynElement fields we pick the largest ByteRange among them. */

impl LargestByteRange for UnprocessedAttribute {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        Some(self.attribute_body.range.byte_range.clone())
    }
}

impl LargestByteRange for UnprocessedTestsMod {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        Some(self.tests_mod_body.range.byte_range.clone())
    }
}

impl LargestByteRange for UnprocessedFunction {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.function_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.function_name.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedMethod {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.method_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.method_name.range.byte_range.clone()));
        best = pick_larger(best, Some(self.impl_body.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedTraitMethodDefinition {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.trait_method_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.method_name.range.byte_range.clone()));
        best = pick_larger(best, Some(self.trait_body.range.byte_range.clone()));
        best = pick_larger(best, Some(self.trait_name.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedTypeAlias {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.type_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.type_name.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedEnum {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.enum_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.enum_name.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedUnion {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.union_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.union_name.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedTrait {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.trait_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.trait_name.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedTraitMethodSignature {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.trait_method_signature.range.byte_range.clone());
        best = pick_larger(best, Some(self.method_signature_name.range.byte_range.clone()));
        best = pick_larger(best, Some(self.trait_body.range.byte_range.clone()));
        best = pick_larger(best, Some(self.trait_name.range.byte_range.clone()));
        best
    }
}

impl LargestByteRange for UnprocessedImpl {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        Some(self.impl_body.range.byte_range.clone())
    }
}

impl LargestByteRange for UnprocessedStruct {
    fn largest_byte_range(&self) -> Option<ByteRange> {
        let mut best = Some(self.struct_body.range.byte_range.clone());
        best = pick_larger(best, Some(self.struct_name.range.byte_range.clone()));
        best
    }
}

/// Represents an unprocessed element with its largest byte range and file location
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementWithRange {
    /// The file path where this element is located
    pub file: PathBuf,
    /// The largest byte range representing this element
    pub byte_range: ByteRange,
    /// Optional identifier for debugging (e.g., "Function: foo", "Struct: Bar")
    pub element_type: String,
}

/// Orders ElementWithRange by start position (descending), then by end position (descending)
/// This ensures we delete from the end of the file first
impl Ord for ElementWithRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by start position (descending - larger start comes first)
        match other.byte_range.start.cmp(&self.byte_range.start) {
            std::cmp::Ordering::Equal => {
                // If starts are equal, compare by end position (descending)
                other.byte_range.end.cmp(&self.byte_range.end)
            }
            ordering => ordering,
        }
    }
}

impl PartialOrd for ElementWithRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// BTreeMap structure organizing elements by file, with elements ordered by range (end to start)
pub type FileRanges = BTreeMap<PathBuf, Vec<ElementWithRange>>;

use std::cmp::Ordering;

/// Represents the nesting relationship between ranges
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NestingRelation {
    /// This range contains the other
    Contains,
    /// This range is contained by the other
    ContainedBy,
    /// Ranges don't overlap or are identical
    NoNesting,
}

impl ElementWithRange {
    /// Check if this range contains another range
    fn contains(&self, other: &Self) -> bool {
        self.byte_range.start <= other.byte_range.start
            && self.byte_range.end >= other.byte_range.end
            && (self.byte_range.start != other.byte_range.start
                || self.byte_range.end != other.byte_range.end)
    }

    /// Determine nesting relationship with another range
    fn nesting_relation(&self, other: &Self) -> NestingRelation {
        if self.contains(other) {
            NestingRelation::Contains
        } else if other.contains(self) {
            NestingRelation::ContainedBy
        } else {
            NestingRelation::NoNesting
        }
    }

    /// Compare for deletion order: innermost first, then by position (end to start)
    fn deletion_order_cmp(&self, other: &Self) -> Ordering {
        match self.nesting_relation(other) {
            // If self contains other, other should come first (delete inner first)
            NestingRelation::Contains => Ordering::Greater,
            // If other contains self, self should come first (delete inner first)
            NestingRelation::ContainedBy => Ordering::Less,
            // No nesting: sort by position (later in file first)
            NestingRelation::NoNesting => {
                // First by start position (descending - later start comes first)
                match other.byte_range.start.cmp(&self.byte_range.start) {
                    Ordering::Equal => {
                        // If starts are equal, by end position (descending)
                        other.byte_range.end.cmp(&self.byte_range.end)
                    }
                    ordering => ordering,
                }
            }
        }
    }
}

/// Sorted container for elements to be deleted from a file
/// Elements are ordered for safe deletion: innermost nested elements first,
/// then by position from end to start of file
#[derive(Debug, Clone)]
pub struct SortedFileRanges {
    /// File path
    pub file: PathBuf,
    /// Elements sorted for safe deletion order
    pub elements: Vec<ElementWithRange>,
}

impl SortedFileRanges {
    /// Create a new SortedFileRanges and sort elements for safe deletion
    pub fn new(file: PathBuf, mut elements: Vec<ElementWithRange>) -> Self {
        elements.sort_by(|a, b| a.deletion_order_cmp(b));
        Self { file, elements }
    }

    /// Check if the ranges are properly ordered for safe deletion
    pub fn validate_ordering(&self) -> Result<(), String> {
        for i in 0..self.elements.len() {
            for j in (i + 1)..self.elements.len() {
                let earlier = &self.elements[i];
                let later = &self.elements[j];

                // If earlier contains later, that's a problem (later should be deleted first)
                if earlier.contains(later) {
                    return Err(format!(
                        "Invalid order: {} at {}..{} is deleted before nested {} at {}..{}",
                        earlier.element_type, earlier.byte_range.start, earlier.byte_range.end,
                        later.element_type, later.byte_range.start, later.byte_range.end
                    ));
                }
            }
        }
        Ok(())
    }
}

/// BTreeMap organizing elements by file with proper deletion ordering
pub type SortedFileRangesMap = BTreeMap<PathBuf, SortedFileRanges>;

/// Convert FileRanges to SortedFileRangesMap with proper deletion ordering
pub fn sort_file_ranges_for_deletion(file_ranges: FileRanges) -> SortedFileRangesMap {
    file_ranges
        .into_iter()
        .map(|(file, elements)| {
            let sorted = SortedFileRanges::new(file.clone(), elements);
            (file, sorted)
        })
        .collect()
}

/// Collect all unprocessed elements into a BTreeMap organized by file,
/// with elements sorted by their byte range (end to start of file)
pub fn collect_file_ranges(all: &AllUnprocessedElements) -> SortedFileRangesMap {
    let mut file_ranges: FileRanges = BTreeMap::new();

    // Helper macro to add an element with its range
    macro_rules! add_element {
        ($file:expr, $range:expr, $type_name:expr) => {
            if let Some(range) = $range {
                let elem = ElementWithRange {
                    file: PathBuf::from($file),
                    byte_range: range,
                    element_type: $type_name.to_string(),
                };
                file_ranges.entry(PathBuf::from($file)).or_insert_with(Vec::new).push(elem);
            }
        };
    }

    // Collect attributes
    for attr in &all.unprocessed_attributes {
        add_element!(&attr.file, attr.largest_byte_range(), "Attribute");
    }

    // Collect test modules
    for test_mod in &all.unprocessed_tests_mods {
        add_element!(&test_mod.file, test_mod.largest_byte_range(), "TestsMod");
    }

    // Collect functions
    for func in &all.unprocessed_functions {
        let type_name = format!("Function: {}", func.function_name.text);
        add_element!(&func.file, func.largest_byte_range(), &type_name);
    }

    // Collect methods
    for method in &all.unprocessed_methods {
        let type_name = format!("Method: {}", method.method_name.text);
        add_element!(&method.file, method.largest_byte_range(), &type_name);
    }

    // Collect impls
    for impl_block in &all.unprocessed_impls {
        add_element!(&impl_block.file, impl_block.largest_byte_range(), "Impl");
    }

    // Collect structs
    for struct_item in &all.unprocessed_structs {
        let type_name = format!("Struct: {}", struct_item.struct_name.text);
        add_element!(&struct_item.file, struct_item.largest_byte_range(), &type_name);
    }

    // Collect traits
    for trait_item in &all.unprocessed_traits {
        let type_name = format!("Trait: {}", trait_item.trait_name.text);
        add_element!(&trait_item.file, trait_item.largest_byte_range(), &type_name);
    }

    // Collect trait method signatures
    for sig in &all.unprocessed_trait_method_sigs {
        let type_name = format!("TraitMethodSig: {}", sig.method_signature_name.text);
        add_element!(&sig.file, sig.largest_byte_range(), &type_name);
    }

    // Collect trait method definitions
    for def in &all.unprocessed_trait_method_defs {
        let type_name = format!("TraitMethodDef: {}", def.method_name.text);
        add_element!(&def.file, def.largest_byte_range(), &type_name);
    }

    // Collect type aliases
    for type_alias in &all.unprocessed_type_aliases {
        let type_name = format!("TypeAlias: {}", type_alias.type_name.text);
        add_element!(&type_alias.file, type_alias.largest_byte_range(), &type_name);
    }

    // Collect enums
    for enum_item in &all.unprocessed_enums {
        let type_name = format!("Enum: {}", enum_item.enum_name.text);
        add_element!(&enum_item.file, enum_item.largest_byte_range(), &type_name);
    }

    // Collect unions
    for union_item in &all.unprocessed_unions {
        let type_name = format!("Union: {}", union_item.union_name.text);
        add_element!(&union_item.file, union_item.largest_byte_range(), &type_name);
    }

    // Convert to sorted map for safe deletion
    sort_file_ranges_for_deletion(file_ranges)
}

/// Delete all ranges from files, processing in safe order (innermost first, end to start)
pub fn delete_file_ranges(root: &Path, file_ranges: SortedFileRangesMap) -> io::Result<()> {
    // Build a set of files under `root` for path resolution
    let mut files_on_disk: BTreeMap<String, PathBuf> = BTreeMap::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let p = entry.path().to_path_buf();
        if let Some(filename) = p.file_name().and_then(|s| s.to_str()) {
            files_on_disk.insert(filename.to_string(), p.clone());
        }
        // Also store the full path
        if let Some(path_str) = p.to_str() {
            files_on_disk.insert(path_str.to_string(), p);
        }
    }

    for (file_key, sorted_ranges) in file_ranges {
        // Validate ordering
        if let Err(e) = sorted_ranges.validate_ordering() {
            eprintln!("Warning: Invalid deletion order for {:?}: {}", file_key, e);
        }

        // Resolve the file path
        let candidate_paths = [
            file_key.clone(),
            root.join(&file_key),
        ];

        let resolved_path = candidate_paths
            .iter()
            .find(|p| p.exists())
            .cloned()
            .or_else(|| {
                // Try to match by filename
                file_key
                    .file_name()
                    .and_then(|s| s.to_str())
                    .and_then(|filename| files_on_disk.get(filename).cloned())
            });

        let path = match resolved_path {
            Some(p) => p,
            None => {
                eprintln!(
                    "Warning: file referenced by unprocessed elements not found: {:?}",
                    file_key
                );
                continue;
            }
        };

        // Read original file bytes
        let mut bytes = fs::read(&path)?;
        let file_len = bytes.len();

        println!("\nProcessing file: {:?}", path);
        println!("  Original size: {} bytes", file_len);
        println!("  Elements to remove: {}", sorted_ranges.elements.len());

        let mut deleted_count = 0;
        let mut deleted_bytes = 0;

        // Elements are already sorted for safe deletion (innermost first, then end to start)
        for elem in sorted_ranges.elements {
            let start = elem.byte_range.start as usize;
            let end = elem.byte_range.end as usize;

            // Validate range
            if start >= end {
                eprintln!("  Skipping invalid range (start >= end): {:?}", elem);
                continue;
            }

            if end > bytes.len() {
                eprintln!(
                    "  Skipping out-of-bounds range for {}: current_len={}, range={}..{}",
                    elem.element_type,
                    bytes.len(),
                    start,
                    end
                );
                continue;
            }

            // Delete the range
            let range_len = end - start;
            bytes.drain(start..end);
            deleted_count += 1;
            deleted_bytes += range_len;

            println!("  ✓ Deleted {} at {}..{} ({} bytes)",
                     elem.element_type, start, end, range_len);
        }

        if deleted_count == 0 {
            println!("  No valid ranges to delete");
            continue;
        }

        // Write back the modified file
        let tmp = path.with_extension("tmp_unprocessed_clean");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &path)?;

        println!("  ✓ Updated file: {:?}", path);
        println!("  Final size: {} bytes (removed {} bytes)", bytes.len(), deleted_bytes);
    }

    Ok(())
}

/// High-level helper: collect all unprocessed elements into a BTreeMap structure,
/// then delete them from files under `root` in the correct order (end to start).
pub fn remove_all_unprocessed(root: &Path, all: &AllUnprocessedElements) -> io::Result<()> {
    let file_ranges = collect_file_ranges(all);

    println!("Collected elements from {} files", file_ranges.len());
    for (file, elements) in &file_ranges {
        println!("  {:?}: {} elements", file, elements.elements.len());
    }

    delete_file_ranges(root, file_ranges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_ordering() {
        let elem1 = ElementWithRange {
            file: PathBuf::from("test.rs"),
            byte_range: ByteRange { start: 100, end: 200 },
            element_type: "Function".to_string(),
        };

        let elem2 = ElementWithRange {
            file: PathBuf::from("test.rs"),
            byte_range: ByteRange { start: 50, end: 80 },
            element_type: "Struct".to_string(),
        };

        let elem3 = ElementWithRange {
            file: PathBuf::from("test.rs"),
            byte_range: ByteRange { start: 150, end: 180 },
            element_type: "Method".to_string(),
        };

        let mut elements = vec![elem1.clone(), elem2.clone(), elem3.clone()];
        elements.sort();

        // Should be ordered: elem3 (150), elem1 (100), elem2 (50)
        assert_eq!(elements[0].byte_range.start, 150);
        assert_eq!(elements[1].byte_range.start, 100);
        assert_eq!(elements[2].byte_range.start, 50);
    }
}
