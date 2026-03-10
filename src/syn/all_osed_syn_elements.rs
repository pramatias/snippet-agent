// all_osed_syn_elements.rs
use crate::AllSynElements;
use crate::syn::osyn_element::{
    OsAttributeBody, OsEnumBody, OsFunctionBody, OsImplBody,
    OsStructBody, OsTestsModBody, OsTraitBody, OsTraitMethodBody,
    OsTraitMethodSignature, OsTypeAliasBody, OsUnionBody,
    OsMethodImplBody,
};
use std::collections::HashSet;
use crate::syn::OsedSynElement;

/// Flat collection of the primary `OsedSynElement` bodies extracted from
/// every element in an `AllSynElements`.  Ownership is moved out of the
/// source so no cloning of the inner `SynElement` text is required.
#[derive(Debug, Default)]
pub struct AllOsedSynElements {
    pub attributes:        Vec<OsAttributeBody>,
    pub methods:           Vec<OsMethodImplBody>,
    pub impls:             Vec<OsImplBody>,
    pub structs:           Vec<OsStructBody>,
    pub traits:            Vec<OsTraitBody>,
    pub functions:         Vec<OsFunctionBody>,
    pub tests_mods:        Vec<OsTestsModBody>,
    pub enums:             Vec<OsEnumBody>,
    pub unions:            Vec<OsUnionBody>,
    pub type_aliases:      Vec<OsTypeAliasBody>,
    pub trait_method_sigs: Vec<OsTraitMethodSignature>,
    pub trait_method_defs: Vec<OsTraitMethodBody>,
}

///from
impl AllOsedSynElements {
    pub fn from(all: AllSynElements) -> Self {
        Self {
            attributes:        all.syn_attributes       .into_iter().map(|x| OsedSynElement::new(x.attribute_body)).collect(),
            methods:           all.syn_methods           .into_iter().map(|x| OsedSynElement::new(x.impl_body)).collect(),
            impls:             all.syn_impls             .into_iter().map(|x| OsedSynElement::new(x.impl_body)).collect(),
            structs:           all.syn_structs           .into_iter().map(|x| OsedSynElement::new(x.struct_body)).collect(),
            traits:            all.syn_traits            .into_iter().map(|x| OsedSynElement::new(x.trait_body)).collect(),
            functions:         all.syn_functions         .into_iter().map(|x| OsedSynElement::new(x.function_body)).collect(),
            tests_mods:        all.syn_tests_mods        .into_iter().map(|x| OsedSynElement::new(x.tests_mod_body)).collect(),
            enums:             all.syn_enums             .into_iter().map(|x| OsedSynElement::new(x.enum_body)).collect(),
            unions:            all.syn_unions            .into_iter().map(|x| OsedSynElement::new(x.union_body)).collect(),
            type_aliases:      all.syn_type_aliases      .into_iter().map(|x| OsedSynElement::new(x.type_body)).collect(),
            trait_method_sigs: all.syn_trait_method_sigs .into_iter().map(|x| OsedSynElement::new(x.trait_method_signature)).collect(),
            trait_method_defs: all.syn_trait_method_defs .into_iter().map(|x| OsedSynElement::new(x.trait_method_body)).collect(),
        }
    }
}

///len
impl AllOsedSynElements {
    /// Total count of all elements across every category.
    pub fn len(&self) -> usize {
        self.attributes.len()
            + self.methods.len()
            + self.impls.len()
            + self.structs.len()
            + self.traits.len()
            + self.functions.len()
            + self.tests_mods.len()
            + self.enums.len()
            + self.unions.len()
            + self.type_aliases.len()
            + self.trait_method_sigs.len()
            + self.trait_method_defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
