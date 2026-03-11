//osyn_element.rs
use crate::syn::syn_element::SynElement;
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange};
use crate::FilePath;
use crate::syn::syn_elements::TypeIdentifiers;
use crate::syn::syn_elements::DSName;

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

#[derive(Debug, Clone)]
pub struct OsedAttribute {
    pub file: FilePath,
    pub attribute_body: OsAttributeBody,
    pub context_lines: String,
}

#[derive(Debug, Clone)]
pub struct OsedMethod {
    pub file: FilePath,
    pub impl_body: OsMethodImplBody,
    pub method_body: OsMethodBody,
    pub method_name: OsMethodName,
    pub impl_signature: OsImplSignature,
    pub function_signature: OsFunctionSignature,
    pub ds_structure: DSName,
    pub type_identifiers: TypeIdentifiers,
}

#[derive(Debug, Clone)]
pub struct OsedImpl {
    pub file: FilePath,
    pub impl_body: OsImplBody,
}

#[derive(Debug, Clone)]
pub struct OsedStruct {
    pub file: FilePath,
    pub struct_body: OsStructBody,
    pub struct_name: OsStructName,
}

#[derive(Debug, Clone)]
pub struct OsedTrait {
    pub file: FilePath,
    pub trait_body: OsTraitBody,
    pub trait_name: OsTraitName,
}

#[derive(Debug, Clone)]
pub struct OsedFunction {
    pub file: FilePath,
    pub function_body: OsFunctionBody,
    pub function_name: OsFunctionName,
}

#[derive(Debug, Clone)]
pub struct OsedTestsMod {
    pub file: FilePath,
    pub tests_mod_body: OsTestsModBody,
}

#[derive(Debug, Clone)]
pub struct OsedEnum {
    pub file: FilePath,
    pub enum_body: OsEnumBody,
    pub enum_name: OsEnumName,
}

#[derive(Debug, Clone)]
pub struct OsedUnion {
    pub file: FilePath,
    pub union_body: OsUnionBody,
    pub union_name: OsUnionName,
}

#[derive(Debug, Clone)]
pub struct OsedTypeAlias {
    pub file: FilePath,
    pub type_body: OsTypeAliasBody,
    pub type_name: OsTypeAliasName,
}

#[derive(Debug, Clone)]
pub struct OsedTraitMethodSig {
    pub file: FilePath,
    pub trait_method_signature: OsTraitMethodSignature,
    pub method_signature_name: OsSignatureName,
    pub trait_body: OsTraitBody,
    pub trait_name: OsTraitName,
}

#[derive(Debug, Clone)]
pub struct OsedTraitMethodDef {
    pub file: FilePath,
    pub trait_body: OsTraitBody,
    pub trait_method_body: OsTraitMethodBody,
    pub method_name: OsTraitMethodName,
    pub trait_name: OsTraitName,
}

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
