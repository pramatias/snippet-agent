//syn_element.rs
use crate::json_selection::raw_elements::Range;
use serde::Deserialize;
use serde::de::IgnoredAny;
use std::sync::Arc;
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange, SynRange};

pub type FilePath = Arc<str>;

#[derive(Debug, Deserialize, Clone)]
pub struct SynElement {
    #[serde(rename = "text")]
    _text: IgnoredAny,
    pub range: SynRange,
}

#[derive(Debug, Clone)]
pub struct SynElementWithText {
    pub range: SynRange,
    pub text: Arc<str>,
}

pub type ImplBody = SynElementWithText;
pub type MethodImplBody = SynElementWithText;
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

impl SynElement {
    /// Resolve the element's text as a slice into the owning file's content.
    /// Pass `file_contents.get(file).map(String::as_str).unwrap_or("")`.
    pub fn text<'a>(&self, file_content: &'a str) -> &'a str {
        let start = self.range.byte_range.start as usize;
        let end = self.range.byte_range.end as usize;
        file_content.get(start..end).unwrap_or("")
    }

    /// Merge two adjacent elements into one spanning both ranges.
    /// The text is the full file slice from self.start to other.end,
    /// which naturally includes any whitespace between them.
    pub fn merge(&self, other: &SynElement) -> SynElement {
        SynElement {
            _text: IgnoredAny,
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

impl Default for SynElement {
    fn default() -> Self {
        SynElement {
            _text: IgnoredAny,
            range: SynRange::default(),
        }
    }
}

impl SynElement {
    /// Construct from an already-converted `SynRange` (e.g. from a `NodeMatch`).
    pub fn from_syn_range(range: SynRange) -> Self {
        Self {
            range,
            _text: Default::default(),
        }
    }
}

///new
impl SynElement {
    /// Construct a range-only element; `_text` is resolved lazily from file
    /// contents and is therefore left empty at this stage.
    pub fn new(range: Range) -> Self {
        Self {
            range: range.into(),
            _text: Default::default(), // Arc<str> / String / PhantomData — whatever _text is
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

impl SynElementWithText {
    pub fn new(range: SynRange, text: impl Into<Arc<str>>) -> Self {
        Self {
            range,
            text: text.into(),
        }
    }

    /// Borrow the pre-stored text directly (no file-contents lookup needed).
    pub fn text_str(&self) -> &str {
        &self.text
    }
}

impl HasByteRange for SynElementWithText {
    fn byte_range(&self) -> &ByteRange {
        &self.range.byte_range
    }
}

pub trait HasPrimaryBody {
    fn primary_body(&self) -> &SynElement;
    fn into_primary_body(self) -> SynElement;
}
