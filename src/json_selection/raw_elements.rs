use serde::Deserialize;

#[derive(Debug)]
pub struct MethodSelection {
    pub file: String,
    pub impl_text: String,
    pub impl_range: Range,

    pub body_text: String,
    pub body_range: Range,

    pub name_text: String,
    pub name_range: Range,
}

#[derive(Debug)]
pub struct TraitMethodDefinitionSelection {
    pub file: String,

    /// enclosing trait (if available)
    pub trait_body_text: String,
    pub trait_body_range: Range,

    /// the method (function_item) inside the trait (with body)
    pub method_body_text: String,
    pub method_body_range: Range,

    /// the method identifier
    pub method_name_text: String,
    pub method_name_range: Range,

    /// the trait's identifier (type_identifier)
    pub trait_name_text: String,
    pub trait_name_range: Range,
}

#[derive(Debug)]
pub struct TypeAliasSelection {
    pub file: String,

    pub body_text: String,
    pub body_range: Range,

    pub name_text: String,
    pub name_range: Range,
}

#[derive(Debug)]
pub struct EnumSelection {
    pub file: String,
    pub enum_body_text: String,
    pub enum_body_range: Range,
    pub enum_name_text: String,
    pub enum_name_range: Range,
}

#[derive(Debug)]
pub struct UnionSelection {
    pub file: String,
    pub union_body_text: String,
    pub union_body_range: Range,
    pub union_name_text: String,
    pub union_name_range: Range,
}

#[derive(Debug)]
pub struct TestsModSelection {
    pub file: String,
    pub tests_mod: String,
    pub range: Range,
}

#[derive(Debug)]
pub struct FunctionSelection {
    pub file: String,
    pub body_text: String,
    pub body_range: Range,
    pub name_text: String,
    pub name_range: Range,
}

#[derive(Debug)]
pub struct TraitSelection {
    pub file: String,
    pub trait_body_text: String,
    pub trait_body_range: Range,
    pub trait_name_text: String,
    pub trait_name_range: Range,
}

#[derive(Debug)]
pub struct TraitMethodSignatureSelection {
    pub file: String,
    // the signature node (function_signature_item)
    pub signature_text: String,
    pub signature_range: Range,
    // the identifier inside the signature
    pub signature_name_text: String,
    pub signature_name_range: Range,
    // enclosing trait (optional — may not be present in every match)
    pub enclosing_trait_text: String,
    pub enclosing_trait_range: Range,
    // trait identifier for the enclosing trait (optional)
    pub trait_name_text: String,
    pub trait_name_range: Range,
}

#[derive(Debug)]
pub struct AttributeSelection {
    pub file: String,
    pub attribute: String,
    pub range: Range,
}

#[derive(Debug)]
pub struct ImplSelection {
    pub file: String,
    pub impl_text: String,
    pub impl_range: Range,
}

#[derive(Debug)]
pub struct StructSelection {
    pub file: String,
    pub body_text: String,
    pub body_range: Range,
    pub name_text: String,
    pub name_range: Range,
}

#[derive(Debug)]
pub struct ModSelection {
    pub file: String,
    pub mod_body_text: String,
    pub mod_body_range: Range,
    pub mod_name_text: String,
    pub mod_name_range: Range,
}

#[derive(Debug)]
pub struct ExpressionStatementSelection {
    pub file: String,
    pub expression_text: String,
    pub expression_range: Range,
}

#[derive(Debug)]
pub struct UseDeclarationSelection {
    pub file: String,
    pub use_text: String,
    pub use_range: Range,
}

#[derive(Debug)]
pub struct MacroDefinitionSelection {
    pub file: String,
    pub macro_body_text: String,
    pub macro_body_range: Range,
    pub macro_name_text: String,
    pub macro_name_range: Range,
}

#[derive(Debug)]
pub struct MacroInvocationSelection {
    pub file: String,
    pub invocation_text: String,
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
