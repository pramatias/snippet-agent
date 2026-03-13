//raw_elements.rs
use serde::Deserialize;
use syntax_queries::byte_range_ordering::*;

#[derive(Debug, Deserialize)]
pub struct MethodSelection {
    pub file: String,
    pub impl_range: Range,
    pub impl_text: String, // ← added
    pub body_range: Range,
    pub name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct ImplSelection {
    pub file: String,
    pub impl_range: Range,
    pub impl_text: String,
}

#[derive(Debug, Deserialize)]
pub struct TraitMethodDefinitionSelection {
    pub file: String,
    pub trait_body_range: Range,
    pub method_body_range: Range,
    pub method_name_range: Range,
    pub trait_name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct TypeAliasSelection {
    pub file: String,
    pub body_range: Range,
    pub name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct EnumSelection {
    pub file: String,
    pub enum_body_range: Range,
    pub enum_name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct UnionSelection {
    pub file: String,
    pub union_body_range: Range,
    pub union_name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct TestsModSelection {
    pub file: String,
    pub range: Range,
}

#[derive(Debug, Deserialize)]
pub struct FunctionSelection {
    pub file: String,
    pub body_range: Range,
    pub name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct TraitSelection {
    pub file: String,
    pub trait_body_range: Range,
    pub trait_name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct TraitMethodSignatureSelection {
    pub file: String,
    pub signature_range: Range,
    pub signature_name_range: Range,
    pub enclosing_trait_range: Range,
    pub trait_name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct AttributeSelection {
    pub file: String,
    pub range: Range,
}

#[derive(Debug, Deserialize)]
pub struct StructSelection {
    pub file: String,
    pub body_range: Range,
    pub name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct ModSelection {
    pub file: String,
    pub mod_body_range: Range,
    pub mod_name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct ExpressionStatementSelection {
    pub file: String,
    pub expression_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct UseDeclarationSelection {
    pub file: String,
    pub use_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct MacroDefinitionSelection {
    pub file: String,
    pub macro_body_range: Range,
    pub macro_name_range: Range,
}

#[derive(Debug, Deserialize)]
pub struct MacroInvocationSelection {
    pub file: String,
    pub invocation_range: Range,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub line: u64,
    pub column: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByteOffset {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub byte_offset: ByteOffset,
    pub start: Position,
    pub end: Position,
}
