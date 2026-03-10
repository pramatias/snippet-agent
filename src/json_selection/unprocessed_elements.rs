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
