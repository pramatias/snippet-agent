use crate::json_selection::unprocessed_elements::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use syntax_queries::{ByteRange, HasByteRange};

pub type AttributeContextLines = String;

pub type ImplSignature = String;
pub type FunctionSignature = String;
pub type DSName = String;

pub type TypeVariable = String;
pub type ConcreteType = String;
pub type TypeSet = HashSet<String>;
pub type CTypeSet = HashSet<ConcreteType>;

pub type TypeVariableMap = HashMap<TypeVariable, TypeSet>;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TypeIdentifiers {
    pub type_variables: Option<TypeVariableMap>,
    pub concrete_types: Option<CTypeSet>,
}

pub type SynTestsMods = UnprocessedTestsMods;
pub type SynFunctions = UnprocessedFunctions;
pub type SynStructs = UnprocessedStructs;
pub type SynTraits = UnprocessedTraits;
pub type SynTraitMethodSigs = UnprocessedTraitMethodSigs;
pub type SynTraitMethodDefs = UnprocessedTraitMethodDefs;
pub type SynTypeAliases = UnprocessedTypeAliases;
pub type SynEnums = UnprocessedEnums;
pub type SynUnions = UnprocessedUnions;
pub type SynImpls = UnprocessedImpls;

#[derive(Debug, Deserialize, Clone)]
pub struct SynMethod {
    pub file: FilePath,

    pub impl_body: MethodImplBody,
    pub method_body: MethodBody,
    pub method_name: MethodName,

    pub impl_signature: ImplSignature,
    pub function_signature: FunctionSignature,
    pub ds_structure: DSName,
    pub type_identifiers: TypeIdentifiers,
}

pub type SynMethods = Vec<SynMethod>;

#[derive(Debug, Clone)]
pub struct SynAttribute {
    pub file: FilePath,

    pub attribute_body: AttributeBody,
    pub context_lines: AttributeContextLines,
}

pub type SynAttributes = Vec<SynAttribute>;

#[derive(Debug, Clone)]
pub struct AllSynElements {
    pub syn_attributes: SynAttributes,
    pub syn_tests_mods: SynTestsMods,
    pub syn_functions: SynFunctions,
    pub syn_methods: SynMethods,
    pub syn_impls: SynImpls,
    pub syn_structs: SynStructs,
    pub syn_traits: SynTraits,
    pub syn_trait_method_sigs: SynTraitMethodSigs,
    pub syn_trait_method_defs: SynTraitMethodDefs,
    pub syn_type_aliases: SynTypeAliases,
    pub syn_enums: SynEnums,
    pub syn_unions: SynUnions,
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

///merge
impl SynElement {
    /// Merge two `SynElement`s whose source regions are adjacent (or
    /// separated only by whitespace).  Text is joined with a newline;
    /// the range spans both elements.
    pub fn merge(&self, other: &SynElement) -> SynElement {
        SynElement {
            text: format!("{}\n{}", self.text, other.text),
            range: self.range.merge(&other.range),
        }
    }
}

///whitespace between
impl SynElement {

    /// Return true when the source region between `self` and `other`
    /// (inside `file_content`) is entirely whitespace.
    ///
    /// Assumes `self` ends before `other` starts (i.e. `self.before(other)`).
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

#[allow(dead_code)]
fn context() {
    const CONTEXT: &str = r#"
pub type UnprocessedAttributes = Vec<UnprocessedAttribute>;
pub type UnprocessedTestsMods = Vec<UnprocessedTestsMod>;
pub type UnprocessedFunctions = Vec<UnprocessedFunction>;
pub type UnprocessedMethods = Vec<UnprocessedMethod>;
pub type UnprocessedImpls = Vec<UnprocessedImpl>;
pub type UnprocessedStructs = Vec<UnprocessedStruct>;
pub type UnprocessedTraits = Vec<UnprocessedTrait>;
pub type UnprocessedTraitMethodSigs = Vec<UnprocessedTraitMethodSignature>;
pub type UnprocessedTraitMethodDefs = Vec<UnprocessedTraitMethodDefinition>;
pub type UnprocessedTypeAliases = Vec<UnprocessedTypeAlias>;
pub type UnprocessedEnums = Vec<UnprocessedEnum>;
pub type UnprocessedUnions = Vec<UnprocessedUnion>;

#[derive(Debug, Clone)]
pub struct AllSynElements {
    pub unprocessed_attributes: UnprocessedAttributes,
    pub unprocessed_tests_mods: UnprocessedTestsMods,
    pub unprocessed_functions: UnprocessedFunctions,
    pub unprocessed_methods: UnprocessedMethods,
    pub unprocessed_impls: UnprocessedImpls,
    pub unprocessed_structs: UnprocessedStructs,
    pub unprocessed_traits: UnprocessedTraits,
    pub unprocessed_trait_method_sigs: UnprocessedTraitMethodSigs,
    pub unprocessed_trait_method_defs: UnprocessedTraitMethodDefs,
    pub unprocessed_type_aliases: UnprocessedTypeAliases,
    pub unprocessed_enums: UnprocessedEnums,
    pub unprocessed_unions: UnprocessedUnions,
}

pub type FilePath = String;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct SynPosition {
    pub line: u64,
    pub column: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct CharactersDimension {
    pub start: SynPosition,
    pub end: SynPosition,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct SynRange {
    pub byte_range: ByteRange,
    pub characters_dimension: CharactersDimension,
}

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

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedMethod {
    pub file: FilePath,

    pub impl_body: MethodImplBody,
    pub method_body: MethodBody,
    pub method_name: MethodName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTraitMethodDefinition {
    pub file: FilePath,

    pub trait_body: TraitBody,
    pub trait_method_body: TraitMethodBody,
    pub method_name: TraitMethodName,
    pub trait_name: TraitName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTypeAlias {
    pub file: FilePath,

    pub type_body: TypeAliasBody,
    pub type_name: TypeAliasName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedEnum {
    pub file: FilePath,

    pub enum_body: EnumBody,
    pub enum_name: EnumName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedUnion {
    pub file: FilePath,

    pub union_body: UnionBody,
    pub union_name: UnionName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTestsMod {
    pub file: FilePath,

    pub tests_mod_body: TestsModBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedFunction {
    pub file: FilePath,

    pub function_body: FunctionBody,
    pub function_name: FunctionName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTrait {
    pub file: FilePath,

    pub trait_body: TraitBody,
    pub trait_name: TraitName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTraitMethodSignature {
    pub file: FilePath,

    pub trait_method_signature: TraitMethodSignature,
    pub method_signature_name: SignatureName,
    pub trait_body: TraitBody,
    pub trait_name: TraitName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedAttribute {
    pub file: FilePath,

    pub attribute_body: AttributeBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedImpl {
    pub file: FilePath,

    pub impl_body: ImplBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedStruct {
    pub file: FilePath,

    pub struct_body: StructBody,
    pub struct_name: StructName,
}

pub trait HasByteRange {
    fn byte_range(&self) -> &ByteRange;

    fn before(&self, other: &impl HasByteRange) -> bool
    fn after(&self, other: &impl HasByteRange) -> bool
    fn contains(&self, other: &impl HasByteRange) -> bool

impl HasByteRange for ByteRange {
    fn byte_range(&self) -> &ByteRange
}

impl HasByteRange for NodeMatch {
    fn byte_range(&self) -> &ByteRange
}

"#;
}
