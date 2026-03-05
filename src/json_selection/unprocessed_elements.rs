use serde::Deserialize;
use syntax_queries::SynRange;

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
}

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

#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedMethod {
    pub file: FilePath,

    pub impl_body: MethodImplBody,
    pub method_body: MethodBody,
    pub method_name: MethodName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTraitMethodDefinition {
    pub file: FilePath,

    pub trait_body: TraitBody,
    pub trait_method_body: TraitMethodBody,
    pub method_name: TraitMethodName,
    pub trait_name: TraitName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTypeAlias {
    pub file: FilePath,

    pub type_body: TypeAliasBody,
    pub type_name: TypeAliasName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedEnum {
    pub file: FilePath,

    pub enum_body: EnumBody,
    pub enum_name: EnumName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedUnion {
    pub file: FilePath,

    pub union_body: UnionBody,
    pub union_name: UnionName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTestsMod {
    pub file: FilePath,

    pub tests_mod_body: TestsModBody,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedFunction {
    pub file: FilePath,

    pub function_body: FunctionBody,
    pub function_name: FunctionName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTrait {
    pub file: FilePath,

    pub trait_body: TraitBody,
    pub trait_name: TraitName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedTraitMethodSignature {
    pub file: FilePath,

    pub trait_method_signature: TraitMethodSignature,
    pub method_signature_name: SignatureName,
    pub trait_body: TraitBody,
    pub trait_name: TraitName,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedAttribute {
    pub file: FilePath,

    pub attribute_body: AttributeBody,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedImpl {
    pub file: FilePath,

    pub impl_body: ImplBody,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct UnprocessedStruct {
    pub file: FilePath,

    pub struct_body: StructBody,
    pub struct_name: StructName,
}
