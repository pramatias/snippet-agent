// file_syn_elements.rs
use crate::AllOsedSynElements;
use crate::AllSynElements;
use crate::FileSynElementTree;
use crate::json_selection::unprocessed_elements::*;
use crate::syn::FilePath;
use crate::syn::osyn_element::*;
use crate::syn::syn_elements::SynAttribute;
use crate::syn::syn_elements::SynMethod;

use std::collections::BTreeMap;
use std::collections::HashSet;
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange};

#[derive(Debug, Clone)]
pub enum AnyFileSynElement {
    Sentinel,
    Attribute(OsedAttribute),
    Method(OsedMethod),
    Impl(OsedImpl),
    Struct(OsedStruct),
    Trait(OsedTrait),
    Function(OsedFunction),
    TestsMod(OsedTestsMod),
    Enum(OsedEnum),
    Union(OsedUnion),
    TypeAlias(OsedTypeAlias),
    TraitMethodSig(OsedTraitMethodSig),
    TraitMethodDef(OsedTraitMethodDef),
}

impl HasByteRange for AnyFileSynElement {
    fn byte_range(&self) -> &ByteRange {
        match self {
            Self::Attribute(x)      => x.attribute_body.byte_range(),
            Self::Method(x)         => x.impl_body.byte_range(),
            Self::Impl(x)           => x.impl_body.byte_range(),
            Self::Struct(x)         => x.struct_body.byte_range(),
            Self::Trait(x)          => x.trait_body.byte_range(),
            Self::Function(x)       => x.function_body.byte_range(),
            Self::TestsMod(x)       => x.tests_mod_body.byte_range(),
            Self::Enum(x)           => x.enum_body.byte_range(),
            Self::Union(x)          => x.union_body.byte_range(),
            Self::TypeAlias(x)      => x.type_body.byte_range(),
            Self::TraitMethodSig(x) => x.trait_method_signature.byte_range(),
            Self::TraitMethodDef(x) => x.trait_method_body.byte_range(),
            Self::Sentinel          => panic!("byte_range called on sentinel root"),
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

///from_file_syn_elements_map
///from_file_syn_elements_map
impl AllOsedSynElements {
    pub fn from_file_syn_elements_map(map: FileSynElementsMap) -> Self {
        let mut out = AllOsedSynElements::default();
        for (_file, fse) in map.0 {
            for el in fse.elements {
                match el {
                    AnyFileSynElement::Attribute(x)      => out.attributes.push(x),
                    AnyFileSynElement::Method(x)         => out.methods.push(x),
                    AnyFileSynElement::Impl(x)           => out.impls.push(x),
                    AnyFileSynElement::Struct(x)         => out.structs.push(x),
                    AnyFileSynElement::Trait(x)          => out.traits.push(x),
                    AnyFileSynElement::Function(x)       => out.functions.push(x),
                    AnyFileSynElement::TestsMod(x)       => out.tests_mods.push(x),
                    AnyFileSynElement::Enum(x)           => out.enums.push(x),
                    AnyFileSynElement::Union(x)          => out.unions.push(x),
                    AnyFileSynElement::TypeAlias(x)      => out.type_aliases.push(x),
                    AnyFileSynElement::TraitMethodSig(x) => out.trait_method_sigs.push(x),
                    AnyFileSynElement::TraitMethodDef(x) => out.trait_method_defs.push(x),
                    AnyFileSynElement::Sentinel          => {}
                }
            }
        }
        out
    }
}

///primary_id
impl AnyFileSynElement {
    pub fn primary_id(&self) -> Option<u64> {
        match self {
            Self::Attribute(x)      => Some(x.attribute_body.byte_range().start),
            Self::Method(x)         => Some(x.impl_body.byte_range().start),
            Self::Impl(x)           => Some(x.impl_body.byte_range().start),
            Self::Struct(x)         => Some(x.struct_body.byte_range().start),
            Self::Trait(x)          => Some(x.trait_body.byte_range().start),
            Self::Function(x)       => Some(x.function_body.byte_range().start),
            Self::TestsMod(x)       => Some(x.tests_mod_body.byte_range().start),
            Self::Enum(x)           => Some(x.enum_body.byte_range().start),
            Self::Union(x)          => Some(x.union_body.byte_range().start),
            Self::TypeAlias(x)      => Some(x.type_body.byte_range().start),
            Self::TraitMethodSig(x) => Some(x.trait_method_signature.byte_range().start),
            Self::TraitMethodDef(x) => Some(x.trait_method_body.byte_range().start),
            Self::Sentinel          => None,
        }
    }
}

// ── Conversion helpers: Syn*/Unprocessed* → Osed* ────────────────────────────

fn attr_to_osed(x: SynAttribute) -> OsedAttribute {
    OsedAttribute {
        file: x.file,
        attribute_body: OsedSynElement::new(x.attribute_body),
        context_lines: x.context_lines,
    }
}

fn method_to_osed(x: SynMethod) -> OsedMethod {
    OsedMethod {
        file: x.file,
        impl_body: OsedSynElement::new(x.impl_body),
        method_body: OsedSynElement::new(x.method_body),
        method_name: OsedSynElement::new(x.method_name),
        impl_signature: OsedSynElement::new(x.impl_signature),
        function_signature: OsedSynElement::new(x.function_signature),
        ds_structure: x.ds_structure,
        type_identifiers: x.type_identifiers,
    }
}

fn impl_to_osed(x: UnprocessedImpl) -> OsedImpl {
    OsedImpl {
        file: x.file,
        impl_body: OsedSynElement::new(x.impl_body),
    }
}

fn struct_to_osed(x: UnprocessedStruct) -> OsedStruct {
    OsedStruct {
        file: x.file,
        struct_body: OsedSynElement::new(x.struct_body),
        struct_name: OsedSynElement::new(x.struct_name),
    }
}

fn trait_to_osed(x: UnprocessedTrait) -> OsedTrait {
    OsedTrait {
        file: x.file,
        trait_body: OsedSynElement::new(x.trait_body),
        trait_name: OsedSynElement::new(x.trait_name),
    }
}

fn function_to_osed(x: UnprocessedFunction) -> OsedFunction {
    OsedFunction {
        file: x.file,
        function_body: OsedSynElement::new(x.function_body),
        function_name: OsedSynElement::new(x.function_name),
    }
}

fn tests_mod_to_osed(x: UnprocessedTestsMod) -> OsedTestsMod {
    OsedTestsMod {
        file: x.file,
        tests_mod_body: OsedSynElement::new(x.tests_mod_body),
    }
}

fn enum_to_osed(x: UnprocessedEnum) -> OsedEnum {
    OsedEnum {
        file: x.file,
        enum_body: OsedSynElement::new(x.enum_body),
        enum_name: OsedSynElement::new(x.enum_name),
    }
}

fn union_to_osed(x: UnprocessedUnion) -> OsedUnion {
    OsedUnion {
        file: x.file,
        union_body: OsedSynElement::new(x.union_body),
        union_name: OsedSynElement::new(x.union_name),
    }
}

fn type_alias_to_osed(x: UnprocessedTypeAlias) -> OsedTypeAlias {
    OsedTypeAlias {
        file: x.file,
        type_body: OsedSynElement::new(x.type_body),
        type_name: OsedSynElement::new(x.type_name),
    }
}

fn trait_method_sig_to_osed(x: UnprocessedTraitMethodSignature) -> OsedTraitMethodSig {
    OsedTraitMethodSig {
        file: x.file,
        trait_method_signature: OsedSynElement::new(x.trait_method_signature),
        method_signature_name: OsedSynElement::new(x.method_signature_name),
        trait_body: OsedSynElement::new(x.trait_body),
        trait_name: OsedSynElement::new(x.trait_name),
    }
}

fn trait_method_def_to_osed(x: UnprocessedTraitMethodDefinition) -> OsedTraitMethodDef {
    OsedTraitMethodDef {
        file: x.file,
        trait_body: OsedSynElement::new(x.trait_body),
        trait_method_body: OsedSynElement::new(x.trait_method_body),
        method_name: OsedSynElement::new(x.method_name),
        trait_name: OsedSynElement::new(x.trait_name),
    }
}

///from_all_syn_elements
impl FileSynElements {
    pub fn from_all_syn_elements(all: &AllSynElements) -> BTreeMap<FilePath, FileSynElements> {
        let mut map: BTreeMap<FilePath, FileSynElements> = BTreeMap::new();

        macro_rules! insert_all {
            ($vec:expr, $variant:expr, $convert:expr) => {
                for el in $vec.iter() {
                    let osed = $convert(el.clone());
                    let file: FilePath = osed.file.clone();
                    let entry = map.entry(file.clone()).or_insert_with(|| FileSynElements {
                        file: file.clone(),
                        elements: Vec::new(),
                    });
                    entry.elements.push($variant(osed));
                }
            };
        }

        insert_all!(&all.syn_attributes,        AnyFileSynElement::Attribute,      attr_to_osed);
        insert_all!(&all.syn_methods,           AnyFileSynElement::Method,         method_to_osed);
        insert_all!(&all.syn_impls,             AnyFileSynElement::Impl,           impl_to_osed);
        insert_all!(&all.syn_structs,           AnyFileSynElement::Struct,         struct_to_osed);
        insert_all!(&all.syn_traits,            AnyFileSynElement::Trait,          trait_to_osed);
        insert_all!(&all.syn_functions,         AnyFileSynElement::Function,       function_to_osed);
        insert_all!(&all.syn_tests_mods,        AnyFileSynElement::TestsMod,       tests_mod_to_osed);
        insert_all!(&all.syn_enums,             AnyFileSynElement::Enum,           enum_to_osed);
        insert_all!(&all.syn_unions,            AnyFileSynElement::Union,          union_to_osed);
        insert_all!(&all.syn_type_aliases,      AnyFileSynElement::TypeAlias,      type_alias_to_osed);
        insert_all!(&all.syn_trait_method_sigs, AnyFileSynElement::TraitMethodSig, trait_method_sig_to_osed);
        insert_all!(&all.syn_trait_method_defs, AnyFileSynElement::TraitMethodDef, trait_method_def_to_osed);

        for fse in map.values_mut() {
            fse.elements.sort_by_key(|e| e.byte_range().start);
        }

        map
    }
}

// macro_rules! impl_syn_element_node {
//     ($t:ty) => {
//         impl SynElementNode for $t {
//             fn file(&self) -> &FilePath {
//                 &self.file
//             }
//         }
//     };
// }

// macro_rules! impl_has_byte_range {
//     ($t:ty, $field:ident) => {
//         impl HasByteRange for $t {
//             fn byte_range(&self) -> &ByteRange {
//                 self.$field.byte_range()
//             }
//         }
//     };
// }

// macro_rules! impl_syn_element {
//     ($t:ty, $field:ident) => {
//         impl_syn_element_node!($t);
//         impl_has_byte_range!($t, $field);
//     };
// }

// impl_syn_element!(SynAttribute, attribute_body);
// impl_syn_element!(SynMethod, impl_body);
// impl_syn_element!(UnprocessedImpl, impl_body);
// impl_syn_element!(UnprocessedStruct, struct_body);
// impl_syn_element!(UnprocessedTrait, trait_body);
// impl_syn_element!(UnprocessedFunction, function_body);
// impl_syn_element!(UnprocessedTestsMod, tests_mod_body);
// impl_syn_element!(UnprocessedEnum, enum_body);
// impl_syn_element!(UnprocessedUnion, union_body);
// impl_syn_element!(UnprocessedTypeAlias, type_body);
// impl_syn_element!(UnprocessedTraitMethodSignature, trait_method_signature);
// impl_syn_element!(UnprocessedTraitMethodDefinition, trait_method_body);
