// file_syn_elements.rs
use crate::AllSynElements;
use crate::FileSynElementTree;
use crate::json_selection::unprocessed_elements::*;
use crate::syn::FilePath;
use crate::syn::syn_elements::SynAttribute;
use crate::syn::syn_elements::SynMethod;
use std::collections::BTreeMap;
use std::collections::HashSet;
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange};

#[derive(Debug, Clone)]
pub enum AnyFileSynElement {
    Sentinel,
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
            Self::Attribute(x) => x.byte_range(),
            Self::Method(x) => x.byte_range(),
            Self::Impl(x) => x.byte_range(),
            Self::Struct(x) => x.byte_range(),
            Self::Trait(x) => x.byte_range(),
            Self::Function(x) => x.byte_range(),
            Self::TestsMod(x) => x.byte_range(),
            Self::Enum(x) => x.byte_range(),
            Self::Union(x) => x.byte_range(),
            Self::TypeAlias(x) => x.byte_range(),
            Self::TraitMethodSig(x) => x.byte_range(),
            Self::TraitMethodDef(x) => x.byte_range(),
            Self::Sentinel => panic!("byte_range called on sentinel root"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileSynElements {
    pub file: FilePath,
    /// Sorted by byte_range().start ascending.
    pub elements: Vec<AnyFileSynElement>,
}

#[derive(Debug, Clone)]
pub struct FileSynElementsMap(pub BTreeMap<FilePath, FileSynElements>);

///from_all_syn_elements
impl FileSynElementsMap {
    pub fn from_all_syn_elements(all: &AllSynElements) -> Self {
        Self(FileSynElements::from_all_syn_elements(all))
    }
}

///filter_by_tree
impl FileSynElementsMap {
    /// For each file node: build tree from indices, prune, replace elements
    /// with survivors in-place. The arena never clones AnyFileSynElement.
    pub fn filter_by_tree(&mut self, max_depth: usize, max_attrs: usize) {
        for fse in self.0.values_mut() {
            fse.elements.sort_by(|a, b| {
                let ra = a.byte_range();
                let rb = b.byte_range();
                ra.start.cmp(&rb.start).then(rb.end.cmp(&ra.end))
            });

            let mut tree = FileSynElementTree::from_file_syn_elements(fse);
            tree.remove_deeper_than(max_depth);
            tree.remove_excess_attributes(max_attrs);

            let surviving: HashSet<usize> = tree.surviving_indices().into_iter().collect();
            drop(tree); // arena freed here — only held usizes

            // Replace in-place: drain old vec, keep only surviving indices
            let old = std::mem::take(&mut fse.elements);
            fse.elements = old
                .into_iter()
                .enumerate()
                .filter(|(i, _)| surviving.contains(i))
                .map(|(_, el)| el)
                .collect();
        }
    }
}

impl AllSynElements {
    pub fn from_file_syn_elements_map(map: FileSynElementsMap) -> Self {
        let mut out = AllSynElements::default();
        for (_file, fse) in map.0 {
            for el in fse.elements {
                match el {
                    AnyFileSynElement::Attribute(x) => out.syn_attributes.push(x),
                    AnyFileSynElement::Method(x) => out.syn_methods.push(x),
                    AnyFileSynElement::Impl(x) => out.syn_impls.push(x),
                    AnyFileSynElement::Struct(x) => out.syn_structs.push(x),
                    AnyFileSynElement::Trait(x) => out.syn_traits.push(x),
                    AnyFileSynElement::Function(x) => out.syn_functions.push(x),
                    AnyFileSynElement::TestsMod(x) => out.syn_tests_mods.push(x),
                    AnyFileSynElement::Enum(x) => out.syn_enums.push(x),
                    AnyFileSynElement::Union(x) => out.syn_unions.push(x),
                    AnyFileSynElement::TypeAlias(x) => out.syn_type_aliases.push(x),
                    AnyFileSynElement::TraitMethodSig(x) => out.syn_trait_method_sigs.push(x),
                    AnyFileSynElement::TraitMethodDef(x) => out.syn_trait_method_defs.push(x),
                    AnyFileSynElement::Sentinel => {}
                }
            }
        }
        out
    }
}

///from all syn elements
impl FileSynElements {
    pub fn from_all_syn_elements(all: &AllSynElements) -> BTreeMap<FilePath, FileSynElements> {
        let mut map: BTreeMap<FilePath, FileSynElements> = BTreeMap::new();

        macro_rules! insert_all {
            ($vec:expr, $variant:expr) => {
                for el in $vec.iter() {
                    let file: &FilePath = &el.file;
                    let entry = map.entry(file.clone()).or_insert_with(|| FileSynElements {
                        file: file.clone(),
                        elements: Vec::new(),
                    });
                    entry.elements.push($variant(el.clone()));
                }
            };
        }

        insert_all!(&all.syn_attributes, AnyFileSynElement::Attribute);
        insert_all!(&all.syn_methods, AnyFileSynElement::Method);
        insert_all!(&all.syn_impls, AnyFileSynElement::Impl);
        insert_all!(&all.syn_structs, AnyFileSynElement::Struct);
        insert_all!(&all.syn_traits, AnyFileSynElement::Trait);
        insert_all!(&all.syn_functions, AnyFileSynElement::Function);
        insert_all!(&all.syn_tests_mods, AnyFileSynElement::TestsMod);
        insert_all!(&all.syn_enums, AnyFileSynElement::Enum);
        insert_all!(&all.syn_unions, AnyFileSynElement::Union);
        insert_all!(&all.syn_type_aliases, AnyFileSynElement::TypeAlias);
        insert_all!(
            &all.syn_trait_method_sigs,
            AnyFileSynElement::TraitMethodSig
        );
        insert_all!(
            &all.syn_trait_method_defs,
            AnyFileSynElement::TraitMethodDef
        );

        for fse in map.values_mut() {
            fse.elements.sort_by_key(|e| e.byte_range().start);
        }

        map
    }
}

///primary_id
impl AnyFileSynElement {
    /// The ID of the primary body `OsedSynElement` for this element variant.
    /// Used to locate the element in `AllSynElements` for invalidation.
    pub fn primary_id(&self) -> Option<u64> {
        match self {
            Self::Attribute(x)       => Some(x.attribute_body.id),
            Self::Method(x)          => Some(x.impl_body.id),
            Self::Impl(x)            => Some(x.impl_body.id),
            Self::Struct(x)          => Some(x.struct_body.id),
            Self::Trait(x)           => Some(x.trait_body.id),
            Self::Function(x)        => Some(x.function_body.id),
            Self::TestsMod(x)        => Some(x.tests_mod_body.id),
            Self::Enum(x)            => Some(x.enum_body.id),
            Self::Union(x)           => Some(x.union_body.id),
            Self::TypeAlias(x)       => Some(x.type_body.id),
            Self::TraitMethodSig(x)  => Some(x.trait_method_signature.id),
            Self::TraitMethodDef(x)  => Some(x.trait_method_body.id),
            Self::Sentinel           => None,
        }
    }
}

#[allow(dead_code)]
pub trait SynElementNode: HasByteRange {
    fn file(&self) -> &FilePath;
}

macro_rules! impl_syn_element_node {
    ($t:ty) => {
        impl SynElementNode for $t {
            fn file(&self) -> &FilePath {
                &self.file
            }
        }
    };
}

macro_rules! impl_has_byte_range {
    ($t:ty, $field:ident) => {
        impl HasByteRange for $t {
            fn byte_range(&self) -> &ByteRange {
                self.$field.byte_range()
            }
        }
    };
}

macro_rules! impl_syn_element {
    ($t:ty, $field:ident) => {
        impl_syn_element_node!($t);
        impl_has_byte_range!($t, $field);
    };
}

impl_syn_element!(SynAttribute, attribute_body);
impl_syn_element!(SynMethod, impl_body);
impl_syn_element!(UnprocessedImpl, impl_body);
impl_syn_element!(UnprocessedStruct, struct_body);
impl_syn_element!(UnprocessedTrait, trait_body);
impl_syn_element!(UnprocessedFunction, function_body);
impl_syn_element!(UnprocessedTestsMod, tests_mod_body);
impl_syn_element!(UnprocessedEnum, enum_body);
impl_syn_element!(UnprocessedUnion, union_body);
impl_syn_element!(UnprocessedTypeAlias, type_body);
impl_syn_element!(UnprocessedTraitMethodSignature, trait_method_signature);
impl_syn_element!(UnprocessedTraitMethodDefinition, trait_method_body);
