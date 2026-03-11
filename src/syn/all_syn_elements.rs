//all_syn_elements.rs
// use crate::AllSynElements;
// use crate::syn::FilePath;
// use crate::syn::file_syn_elements::AnyFileSynElement;
// // use crate::syn::syn_elements::{SynAttribute, SynMethod};
// use crate::FileSynElementTree;
// use std::collections::BTreeMap;

// impl AllSynElements {
//     pub fn from_file_syn_elements_map(map: FileSynElementsMap) -> Self {
//         let mut out = AllSynElements::default();
//         for (_file, fse) in map.0 {
//             for el in fse.elements {
//                 match el {
//                     AnyFileSynElement::Attribute(x)          => out.syn_attributes.push(x),
//                     AnyFileSynElement::Method(x)             => out.syn_methods.push(x),
//                     AnyFileSynElement::Impl(x)               => out.syn_impls.push(x),
//                     AnyFileSynElement::Struct(x)             => out.syn_structs.push(x),
//                     AnyFileSynElement::Trait(x)              => out.syn_traits.push(x),
//                     AnyFileSynElement::Function(x)           => out.syn_functions.push(x),
//                     AnyFileSynElement::TestsMod(x)           => out.syn_tests_mods.push(x),
//                     AnyFileSynElement::Enum(x)               => out.syn_enums.push(x),
//                     AnyFileSynElement::Union(x)              => out.syn_unions.push(x),
//                     AnyFileSynElement::TypeAlias(x)          => out.syn_type_aliases.push(x),
//                     AnyFileSynElement::TraitMethodSig(x)     => out.syn_trait_method_sigs.push(x),
//                     AnyFileSynElement::TraitMethodDef(x)     => out.syn_trait_method_defs.push(x),
//                     AnyFileSynElement::Sentinel              => {}
//                 }
//             }
//         }
//         out
//     }
// }
