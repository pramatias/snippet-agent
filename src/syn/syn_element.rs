//syn_element.rs
use serde::Deserialize;
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange, SynRange};

pub type FilePath = String;

#[derive(Debug, Deserialize, Clone)]
pub struct SynElement {
    pub text: String,
    #[allow(dead_code)]
    pub range: SynRange,
}

pub type MethodImplBody = SynElement;
pub type MethodBody = SynElement;
pub type MethodName = SynElement;
pub type TraitBody = SynElement;
pub type TraitMethodBody = SynElement;
pub type TraitMethodName = SynElement;
pub type TraitName = SynElement;
pub type TypeAliasBody = SynElement;
pub type TypeAliasName = SynElement;
pub type EnumBody = SynElement;
pub type EnumName = SynElement;
pub type UnionBody = SynElement;
pub type UnionName = SynElement;
pub type TestsModBody = SynElement;
pub type FunctionBody = SynElement;
pub type FunctionName = SynElement;
pub type TraitMethodSignature = SynElement;
pub type SignatureName = SynElement;
pub type AttributeBody = SynElement;
pub type ImplBody = SynElement;
pub type StructBody = SynElement;
pub type StructName = SynElement;
pub type ModBody = SynElement;
pub type ModName = SynElement;
pub type ExpressionStatementBody = SynElement;
pub type UseDeclarationBody = SynElement;
pub type MacroDefinitionBody = SynElement;
pub type MacroDefinitionName = SynElement;
pub type MacroInvocationBody = SynElement;
pub type ImplSignature = SynElement;
pub type FunctionSignature = SynElement;

impl Default for SynElement {
    fn default() -> Self {
        SynElement {
            text: String::new(),
            range: SynRange::default(),
        }
    }
}

impl HasByteRange for SynElement {
    fn byte_range(&self) -> &ByteRange {
        &self.range.byte_range
    }
}

impl HasByteRange for &SynElement {
    fn byte_range(&self) -> &ByteRange {
        &self.range.byte_range
    }
}

//merge and only whitespace in between
impl SynElement {
    pub fn merge(&self, other: &SynElement) -> SynElement {
        SynElement {
            text: format!("{}\n{}", self.text, other.text),
            range: self.range.merge(&other.range),
        }
    }

    pub fn only_whitespace_between(&self, other: &SynElement, file_content: &str) -> bool {
        let start = self.range.byte_range.end as usize;
        let end = other.range.byte_range.start as usize;
        if start > end {
            return false;
        }
        file_content
            .get(start..end)
            .map(|gap| gap.chars().all(char::is_whitespace))
            .unwrap_or(false)
    }
}

impl std::fmt::Display for SynElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
