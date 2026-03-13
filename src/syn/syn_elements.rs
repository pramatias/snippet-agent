//syn_elements.rs
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_element::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// ── Type enrichment types ─────────────────────────────────────────────────────
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

// ── Pass-through aliases ──────────────────────────────────────────────────────
pub type SynTestsMods = Vec<Arc<UnprocessedTestsMod>>;
pub type SynFunctions = Vec<Arc<UnprocessedFunction>>;
pub type SynStructs = Vec<Arc<UnprocessedStruct>>;
pub type SynTraits = Vec<Arc<UnprocessedTrait>>;
pub type SynTraitMethodSigs = Vec<Arc<UnprocessedTraitMethodSignature>>;
pub type SynTraitMethodDefs = Vec<Arc<UnprocessedTraitMethodDefinition>>;
pub type SynTypeAliases = Vec<Arc<UnprocessedTypeAlias>>;
pub type SynEnums = Vec<Arc<UnprocessedEnum>>;
pub type SynUnions = Vec<Arc<UnprocessedUnion>>;
pub type SynImpls = Vec<Arc<UnprocessedImpl>>;

// ── Enriched syn structs ──────────────────────────────────────────────────────

// Deserialize intentionally omitted: impl_body is SynElementWithText which
// is constructed from AST grep output, not deserialized from JSON.
#[derive(Debug, Clone)]
pub struct SynMethod {
    pub file: FilePath,
    pub impl_body: MethodImplBody, // SynElementWithText
    pub method_body: MethodBody,
    pub method_name: MethodName,
    pub impl_signature: ImplSignature,
    pub function_signature: FunctionSignature,
    pub ds_structure: DSName,
    pub type_identifiers: TypeIdentifiers,
}

#[derive(Debug, Clone)]
pub struct SynAttribute {
    pub file: FilePath,
    pub attribute_body: AttributeBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SynMod {
    pub file: FilePath,
    pub mod_name: ModName,
    pub mod_body: ModBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SynExpressionStatement {
    pub file: FilePath,
    pub expression_body: ExpressionStatementBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SynUseDeclaration {
    pub file: FilePath,
    pub use_body: UseDeclarationBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SynMacroDefinition {
    pub file: FilePath,
    pub macro_definition_name: MacroDefinitionName,
    pub macro_definition_body: MacroDefinitionBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SynMacroInvocation {
    pub file: FilePath,
    pub invocation_body: MacroInvocationBody,
}

// ── Collection aliases ────────────────────────────────────────────────────────
pub type SynMethods = Vec<Arc<SynMethod>>;
pub type SynAttributes = Vec<Arc<SynAttribute>>;
pub type SynMods = Vec<Arc<SynMod>>;
pub type SynExpressionStatements = Vec<Arc<SynExpressionStatement>>;
pub type SynUseDeclarations = Vec<Arc<SynUseDeclaration>>;
pub type SynMacroDefinitions = Vec<Arc<SynMacroDefinition>>;
pub type SynMacroInvocations = Vec<Arc<SynMacroInvocation>>;

// ── HasPrimaryBody — for range-only SynElement fields ────────────────────────
macro_rules! impl_primary_body {
    ($t:ty, $field:ident) => {
        impl HasPrimaryBody for $t {
            fn primary_body(&self) -> &SynElement {
                &self.$field
            }
            fn into_primary_body(self) -> SynElement {
                self.$field
            }
        }
    };
}

impl_primary_body!(SynAttribute, attribute_body);
impl_primary_body!(UnprocessedStruct, struct_body);
impl_primary_body!(UnprocessedTrait, trait_body);
impl_primary_body!(UnprocessedFunction, function_body);
impl_primary_body!(UnprocessedTestsMod, tests_mod_body);
impl_primary_body!(UnprocessedEnum, enum_body);
impl_primary_body!(UnprocessedUnion, union_body);
impl_primary_body!(UnprocessedTypeAlias, type_body);
impl_primary_body!(UnprocessedTraitMethodSignature, trait_method_signature);
impl_primary_body!(UnprocessedTraitMethodDefinition, trait_method_body);

// ── HasPrimaryBodyWithText — for SynElementWithText impl_body fields ─────────
//
// SynMethod and UnprocessedImpl carry their impl text inside the body itself
// (captured at AST grep time) so they cannot satisfy HasPrimaryBody whose
// return type is &SynElement.  Use this trait instead when the text is needed.
pub trait HasPrimaryBodyWithText {
    fn primary_body_with_text(&self) -> &SynElementWithText;
    fn into_primary_body_with_text(self) -> SynElementWithText;
}

macro_rules! impl_primary_body_with_text {
    ($t:ty, $field:ident) => {
        impl HasPrimaryBodyWithText for $t {
            fn primary_body_with_text(&self) -> &SynElementWithText {
                &self.$field
            }
            fn into_primary_body_with_text(self) -> SynElementWithText {
                self.$field
            }
        }
    };
}

impl_primary_body_with_text!(SynMethod, impl_body);
impl_primary_body_with_text!(UnprocessedImpl, impl_body);

// ── Aggregate ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Default)]
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
    pub syn_mods: SynMods,
    pub syn_expression_statements: SynExpressionStatements,
    pub syn_use_declarations: SynUseDeclarations,
    pub syn_macro_definitions: SynMacroDefinitions,
    pub syn_macro_invocations: SynMacroInvocations,
}

#[allow(dead_code)]
fn context() {
    const CONTEXT: &str = r#"
//unprocessed elements.rs
use crate::syn::syn_element::*;
use serde::Deserialize;

// ── Collection type aliases ───────────────────────────────────────────────────
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
pub type UnprocessedMods = Vec<UnprocessedMod>;
pub type UnprocessedExpressionStatements = Vec<UnprocessedExpressionStatement>;
pub type UnprocessedUseDeclarations = Vec<UnprocessedUseDeclaration>;
pub type UnprocessedMacroDefinitions = Vec<UnprocessedMacroDefinition>;
pub type UnprocessedMacroInvocations = Vec<UnprocessedMacroInvocation>;

// ── Aggregate ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct AllUnprocessedElements {
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
    pub unprocessed_mods: UnprocessedMods,
    pub unprocessed_expression_statements: UnprocessedExpressionStatements,
    pub unprocessed_use_declarations: UnprocessedUseDeclarations,
    pub unprocessed_macro_definitions: UnprocessedMacroDefinitions,
    pub unprocessed_macro_invocations: UnprocessedMacroInvocations,
}

// ── Structs ───────────────────────────────────────────────────────────────────
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

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedMod {
    pub file: FilePath,
    pub mod_body: ModBody,
    pub mod_name: ModName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedExpressionStatement {
    pub file: FilePath,
    pub expression_body: ExpressionStatementBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedUseDeclaration {
    pub file: FilePath,
    pub use_body: UseDeclarationBody,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedMacroDefinition {
    pub file: FilePath,
    pub macro_body: MacroDefinitionBody,
    pub macro_name: MacroDefinitionName,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedMacroInvocation {
    pub file: FilePath,
    pub invocation_body: MacroInvocationBody,
}

//byte_range_ordering.rs
use serde::Deserialize;

pub trait HasByteRange {
    fn byte_range(&self) -> &ByteRange;

    fn before(&self, other: &impl HasByteRange) -> bool {
        self.byte_range().before(other.byte_range())
    }
    fn after(&self, other: &impl HasByteRange) -> bool {
        self.byte_range().after(other.byte_range())
    }
    fn contains(&self, other: &impl HasByteRange) -> bool {
        self.byte_range().contains(other.byte_range())
    }

    /// All items whose range ends at or before `divider` starts.
    fn before_all<'a, T: HasByteRange>(items: &'a [T], divider: &impl HasByteRange) -> Vec<&'a T> {
        let divider_start = divider.byte_range().start;
        // Items with start >= divider_start have end >= start >= divider_start,
        // so they can never satisfy `end <= divider_start`.
        let hi = items.partition_point(|x| x.byte_range().start < divider_start);
        items[..hi]
            .iter()
            .filter(|x| x.byte_range().end <= divider_start)
            .collect()
    }

    /// All items whose range starts at or after `divider` ends.
    fn after_all<'a, T: HasByteRange>(items: &'a [T], divider: &impl HasByteRange) -> Vec<&'a T> {
        let divider_end = divider.byte_range().end;
        let lo = items.partition_point(|x| x.byte_range().start < divider_end);
        items[lo..].iter().collect()
    }

    /// The item immediately before `limit` (ends earliest while still before `limit`).
    fn immediate_before<'a, T: HasByteRange>(
        items: &'a [T],
        limit: &impl HasByteRange,
    ) -> Option<&'a T> {
        let limit_start = limit.byte_range().start;
        let hi = items.partition_point(|x| x.byte_range().start < limit_start);
        items[..hi]
            .iter()
            .filter(|x| x.byte_range().end <= limit_start)
            .max_by_key(|x| x.byte_range().end)
    }

    /// The item immediately after `limit` (starts latest while still after `limit`).
    fn immediate_after<'a, T: HasByteRange>(
        items: &'a [T],
        limit: &impl HasByteRange,
    ) -> Option<&'a T> {
        let limit_end = limit.byte_range().end;
        let lo = items.partition_point(|x| x.byte_range().start < limit_end);
        // Slice is sorted by start, so the first element at `lo` has the minimum start.
        items.get(lo)
    }

    /// The first (earliest-start) item fully contained within `container`.
    fn first_contained<'a, T: HasByteRange>(
        items: &'a [T],
        container: &impl HasByteRange,
    ) -> Option<&'a T> {
        let range = container.byte_range();
        let lo = items.partition_point(|x| x.byte_range().start < range.start);
        items[lo..]
            .iter()
            // Once start exceeds container.end nothing further can be contained.
            .take_while(|x| x.byte_range().start <= range.end)
            .find(|x| x.byte_range().end <= range.end)
    }

    /// The last (latest-start) item fully contained within `container`.
    fn last_contained<'a, T: HasByteRange>(
        items: &'a [T],
        container: &impl HasByteRange,
    ) -> Option<&'a T> {
        let range = container.byte_range();
        let lo = items.partition_point(|x| x.byte_range().start < range.start);
        let hi = items.partition_point(|x| x.byte_range().start <= range.end);
        items[lo..hi]
            .iter()
            .filter(|x| x.byte_range().end <= range.end)
            .last()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct SynPosition {
    pub line: u64,
    pub column: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct CharactersDimension {
    pub start: SynPosition,
    pub end: SynPosition,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct SynRange {
    pub byte_range: ByteRange,
    pub characters_dimension: CharactersDimension,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct NodeMatch {
    pub text: String,

    #[allow(dead_code)]
    pub range: SynRange,
}

impl ByteRange {
    /// Self ends before other starts (no overlap)
    pub fn before(&self, other: &ByteRange) -> bool {
        self.end <= other.start
    }

    /// Self starts after other ends (no overlap)
    pub fn after(&self, other: &ByteRange) -> bool {
        self.start >= other.end
    }

    /// Self fully contains other
    pub fn contains(&self, other: &ByteRange) -> bool {
        self.start <= other.start && self.end >= other.end
    }
}

impl HasByteRange for ByteRange {
    fn byte_range(&self) -> &ByteRange {
        self
    }
}

impl HasByteRange for NodeMatch {
    fn byte_range(&self) -> &ByteRange {
        &self.range.byte_range
    }
}

impl HasByteRange for &NodeMatch {
    fn byte_range(&self) -> &ByteRange {
        &self.range.byte_range
    }
}

impl SynRange {
    /// Merge two ranges so that the result spans from the earlier start to the
    /// later end.  The `characters_dimension` is merged the same way.
    pub fn merge(&self, other: &SynRange) -> SynRange {
        SynRange {
            byte_range: self.byte_range.merge(&other.byte_range),
            characters_dimension: self
                .characters_dimension
                .merge(&other.characters_dimension),
        }
    }
}

impl ByteRange {
    /// Produce a range that spans both inputs.
    pub fn merge(&self, other: &ByteRange) -> ByteRange {
        ByteRange {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl CharactersDimension {
    /// Produce a dimension that spans both inputs (earliest start, latest end).
    pub fn merge(&self, other: &CharactersDimension) -> CharactersDimension {
        // "earlier" start = smaller (line, column) pair
        let start = if (self.start.line, self.start.column)
            <= (other.start.line, other.start.column)
        {
            self.start.clone()
        } else {
            other.start.clone()
        };

        // "later" end = larger (line, column) pair
        let end =
            if (self.end.line, self.end.column) >= (other.end.line, other.end.column) {
                self.end.clone()
            } else {
                other.end.clone()
            };

        CharactersDimension { start, end }
    }
}

"#;
}
