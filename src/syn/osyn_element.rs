//osyn_element.rs
use crate::syn::syn_element::SynElement;
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange};

static NEXT_OSYN_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_OSYN_ID.fetch_add(1, Ordering::Relaxed)
}

/// A `SynElement` with a stable numeric identity and a validity flag.
/// The `id` is assigned at construction/deserialization time and never changes.
/// `valid = false` means the element has been pruned by the tree
#[derive(Debug, Clone)]
pub struct OsedSynElement {
    pub id: u64,
    pub valid: bool,
    pub inner: SynElement,
}

pub type OsMethodImplBody = OsedSynElement;
pub type OsMethodBody = OsedSynElement;
pub type OsMethodName = OsedSynElement;
pub type OsTraitBody = OsedSynElement;
pub type OsTraitMethodBody = OsedSynElement;
pub type OsTraitMethodName = OsedSynElement;
pub type OsTraitName = OsedSynElement;
pub type OsTypeAliasBody = OsedSynElement;
pub type OsTypeAliasName = OsedSynElement;
pub type OsEnumBody = OsedSynElement;
pub type OsEnumName = OsedSynElement;
pub type OsUnionBody = OsedSynElement;
pub type OsUnionName = OsedSynElement;
pub type OsTestsModBody = OsedSynElement;
pub type OsFunctionBody = OsedSynElement;
pub type OsFunctionName = OsedSynElement;
pub type OsTraitMethodSignature = OsedSynElement;
pub type OsSignatureName = OsedSynElement;
pub type OsAttributeBody = OsedSynElement;
pub type OsImplBody = OsedSynElement;
pub type OsStructBody = OsedSynElement;
pub type OsStructName = OsedSynElement;
pub type OsModBody = OsedSynElement;
pub type OsModName = OsedSynElement;
pub type OsExpressionStatementBody = OsedSynElement;
pub type OsUseDeclarationBody = OsedSynElement;
pub type OsMacroDefinitionBody = OsedSynElement;
pub type OsMacroDefinitionName = OsedSynElement;
pub type OsMacroInvocationBody = OsedSynElement;
pub type OsImplSignature = OsedSynElement;
pub type OsFunctionSignature = OsedSynElement;

impl OsedSynElement {
    pub fn new(inner: SynElement) -> Self {
        Self {
            id: next_id(),
            valid: true,
            inner,
        }
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }
}

/// Deserialize from the same JSON shape as `SynElement`; `id` is assigned
/// automatically and `valid` starts as `true`.
impl<'de> Deserialize<'de> for OsedSynElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = SynElement::deserialize(deserializer)?;
        Ok(OsedSynElement::new(inner))
    }
}

impl HasByteRange for OsedSynElement {
    fn byte_range(&self) -> &ByteRange {
        self.inner.byte_range()
    }
}

impl HasByteRange for &OsedSynElement {
    fn byte_range(&self) -> &ByteRange {
        self.inner.byte_range()
    }
}

impl std::fmt::Display for OsedSynElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Default for OsedSynElement {
    fn default() -> Self {
        OsedSynElement::new(SynElement::default())
    }
}
