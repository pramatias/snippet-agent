// file_syn_elements.rs
use crate::json_selection::unprocessed_elements::*;
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange};
use std::collections::BTreeMap;
use crate::syn::syn_elements::SynAttribute;
use crate::syn::syn_elements::SynMethod;
use crate::AllSynElements;
use crate::syn::FilePath;

#[derive(Debug, Clone)]
pub enum AnyFileSynElement {
    Attribute(SynAttribute),
    Method(SynMethod),
    Impl(UnprocessedImpl),
    Struct(UnprocessedStruct),
    Trait(UnprocessedTrait),
    Function(UnprocessedFunction),
    TestsMod(UnprocessedTestsMod),
    Enum(UnprocessedEnum),
    Union(UnprocessedUnion),
    TypeAlias(UnprocessedTypeAlias),
    TraitMethodSig(UnprocessedTraitMethodSignature),
    TraitMethodDef(UnprocessedTraitMethodDefinition),
}

impl HasByteRange for AnyFileSynElement {
    fn byte_range(&self) -> &ByteRange {
        match self {
            Self::Attribute(x)      => x.byte_range(),
            Self::Method(x)         => x.byte_range(),
            Self::Impl(x)           => x.byte_range(),
            Self::Struct(x)         => x.byte_range(),
            Self::Trait(x)          => x.byte_range(),
            Self::Function(x)       => x.byte_range(),
            Self::TestsMod(x)       => x.byte_range(),
            Self::Enum(x)           => x.byte_range(),
            Self::Union(x)          => x.byte_range(),
            Self::TypeAlias(x)      => x.byte_range(),
            Self::TraitMethodSig(x) => x.byte_range(),
            Self::TraitMethodDef(x) => x.byte_range(),
        }
    }
}

/// All syn elements from a single file, sorted ascending by byte-range start.
/// No file I/O — purely an ordering structure.
#[derive(Debug, Clone)]
pub struct FileSynElements {
    pub file: FilePath,
    /// Sorted by byte_range().start ascending.
    pub elements: Vec<AnyFileSynElement>,
}

///from all syn elements
impl FileSynElements {
    pub fn from_all_syn_elements(all: &AllSynElements) -> BTreeMap<FilePath, FileSynElements> {
        let mut map: BTreeMap<FilePath, FileSynElements> = BTreeMap::new();

        macro_rules! insert_all {
            ($vec:expr, $variant:expr) => {
                for el in $vec.iter() {
                    let file: &FilePath = &el.file;
                    let entry = map
                        .entry(file.clone())
                        .or_insert_with(|| FileSynElements {
                            file: file.clone(),
                            elements: Vec::new(),
                        });
                    entry.elements.push($variant(el.clone()));
                }
            };
        }

        insert_all!(&all.syn_attributes,        AnyFileSynElement::Attribute);
        insert_all!(&all.syn_methods,            AnyFileSynElement::Method);
        insert_all!(&all.syn_impls,              AnyFileSynElement::Impl);
        insert_all!(&all.syn_structs,            AnyFileSynElement::Struct);
        insert_all!(&all.syn_traits,             AnyFileSynElement::Trait);
        insert_all!(&all.syn_functions,          AnyFileSynElement::Function);
        insert_all!(&all.syn_tests_mods,         AnyFileSynElement::TestsMod);
        insert_all!(&all.syn_enums,              AnyFileSynElement::Enum);
        insert_all!(&all.syn_unions,             AnyFileSynElement::Union);
        insert_all!(&all.syn_type_aliases,       AnyFileSynElement::TypeAlias);
        insert_all!(&all.syn_trait_method_sigs,  AnyFileSynElement::TraitMethodSig);
        insert_all!(&all.syn_trait_method_defs,  AnyFileSynElement::TraitMethodDef);

        for fse in map.values_mut() {
            fse.elements.sort_by_key(|e| e.byte_range().start);
        }

        map
    }
}

pub trait SynElementNode: HasByteRange {
    fn file(&self) -> &FilePath;
}

macro_rules! impl_syn_element_node {
    ($t:ty) => {
        impl SynElementNode for $t {
            fn file(&self) -> &FilePath { &self.file }
        }
    };
}

macro_rules! impl_has_byte_range {
    ($t:ty, $field:ident) => {
        impl HasByteRange for $t {
            fn byte_range(&self) -> &ByteRange { self.$field.byte_range() }
        }
    };
}

/// Convenience macro that expands both impls in one call.
macro_rules! impl_syn_element {
    ($t:ty, $field:ident) => {
        impl_syn_element_node!($t);
        impl_has_byte_range!($t, $field);
    };
}

impl_syn_element!(SynAttribute,                   attribute_body);
impl_syn_element!(SynMethod,                      impl_body);
impl_syn_element!(UnprocessedImpl,                impl_body);
impl_syn_element!(UnprocessedStruct,              struct_body);
impl_syn_element!(UnprocessedTrait,               trait_body);
impl_syn_element!(UnprocessedFunction,            function_body);
impl_syn_element!(UnprocessedTestsMod,            tests_mod_body);
impl_syn_element!(UnprocessedEnum,                enum_body);
impl_syn_element!(UnprocessedUnion,               union_body);
impl_syn_element!(UnprocessedTypeAlias,           type_body);
impl_syn_element!(UnprocessedTraitMethodSignature, trait_method_signature);
impl_syn_element!(UnprocessedTraitMethodDefinition, trait_method_body);
