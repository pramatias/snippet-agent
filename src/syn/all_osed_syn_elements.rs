// all_osed_syn_elements.rs
use crate::AllSynElements;
use crate::syn::syn_element::HasPrimaryBody;

// // all_osed_syn_elements.rs
// use crate::AllSynElements;
use crate::syn::osyn_element::{
    OsedAttribute, OsedEnum, OsedFunction, OsedImpl, OsedMethod,
    OsedStruct, OsedTestsMod, OsedTrait, OsedTraitMethodDef,
    OsedTraitMethodSig, OsedTypeAlias, OsedUnion, OsedSynElement,
};

fn osed(el: crate::syn::syn_element::SynElement) -> OsedSynElement {
    OsedSynElement::new(el)
}

/// Flat collection of fully-contextual `Osed*` elements extracted from every
/// element in an `AllSynElements`.  Ownership is moved out of the source so
/// no cloning of the inner `SynElement` text is required.
#[derive(Debug, Default)]
pub struct AllOsedSynElements {
    pub attributes:        Vec<OsedAttribute>,
    pub methods:           Vec<OsedMethod>,
    pub impls:             Vec<OsedImpl>,
    pub structs:           Vec<OsedStruct>,
    pub traits:            Vec<OsedTrait>,
    pub functions:         Vec<OsedFunction>,
    pub tests_mods:        Vec<OsedTestsMod>,
    pub enums:             Vec<OsedEnum>,
    pub unions:            Vec<OsedUnion>,
    pub type_aliases:      Vec<OsedTypeAlias>,
    pub trait_method_sigs: Vec<OsedTraitMethodSig>,
    pub trait_method_defs: Vec<OsedTraitMethodDef>,
}

///from
impl AllOsedSynElements {
    pub fn from(all: AllSynElements) -> Self {
        Self {
            attributes: all.syn_attributes.into_iter().map(|x| OsedAttribute {
                file:           x.file,
                attribute_body: osed(x.attribute_body),
                context_lines:  x.context_lines,
            }).collect(),

            methods: all.syn_methods.into_iter().map(|x| OsedMethod {
                file:               x.file,
                impl_body:          osed(x.impl_body),
                method_body:        osed(x.method_body),
                method_name:        osed(x.method_name),
                impl_signature:     osed(x.impl_signature),
                function_signature: osed(x.function_signature),
                ds_structure:       x.ds_structure,
                type_identifiers:   x.type_identifiers,
            }).collect(),

            impls: all.syn_impls.into_iter().map(|x| OsedImpl {
                file:      x.file,
                impl_body: osed(x.impl_body),
            }).collect(),

            structs: all.syn_structs.into_iter().map(|x| OsedStruct {
                file:        x.file,
                struct_body: osed(x.struct_body),
                struct_name: osed(x.struct_name),
            }).collect(),

            traits: all.syn_traits.into_iter().map(|x| OsedTrait {
                file:       x.file,
                trait_body: osed(x.trait_body),
                trait_name: osed(x.trait_name),
            }).collect(),

            functions: all.syn_functions.into_iter().map(|x| OsedFunction {
                file:          x.file,
                function_body: osed(x.function_body),
                function_name: osed(x.function_name),
            }).collect(),

            tests_mods: all.syn_tests_mods.into_iter().map(|x| OsedTestsMod {
                file:           x.file,
                tests_mod_body: osed(x.tests_mod_body),
            }).collect(),

            enums: all.syn_enums.into_iter().map(|x| OsedEnum {
                file:      x.file,
                enum_body: osed(x.enum_body),
                enum_name: osed(x.enum_name),
            }).collect(),

            unions: all.syn_unions.into_iter().map(|x| OsedUnion {
                file:       x.file,
                union_body: osed(x.union_body),
                union_name: osed(x.union_name),
            }).collect(),

            type_aliases: all.syn_type_aliases.into_iter().map(|x| OsedTypeAlias {
                file:      x.file,
                type_body: osed(x.type_body),
                type_name: osed(x.type_name),
            }).collect(),

            trait_method_sigs: all.syn_trait_method_sigs.into_iter().map(|x| OsedTraitMethodSig {
                file:                   x.file,
                trait_method_signature: osed(x.trait_method_signature),
                method_signature_name:  osed(x.method_signature_name),
                trait_body:             osed(x.trait_body),
                trait_name:             osed(x.trait_name),
            }).collect(),

            trait_method_defs: all.syn_trait_method_defs.into_iter().map(|x| OsedTraitMethodDef {
                file:              x.file,
                trait_body:        osed(x.trait_body),
                trait_method_body: osed(x.trait_method_body),
                method_name:       osed(x.method_name),
                trait_name:        osed(x.trait_name),
            }).collect(),
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

///print
fn print_header(label: &str, count: usize) {
    println!("\n=== {label} ({count}) ===");
}

fn print_osed_element(label: &str, el: &OsedSynElement, indent: usize) {
    let pad = "  ".repeat(indent);
    let preview = el.inner.text.lines().take(2).collect::<Vec<_>>().join("\n");
    println!("{pad}{label} [id={}]:", el.id);
    println!("{pad}  preview: {:?}", preview);
}

impl AllOsedSynElements {
    pub fn print_attributes(&self) {
        print_header("attributes", self.attributes.len());
        for (i, item) in self.attributes.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("attribute_body", &item.attribute_body, 2);
        }
    }

    pub fn print_methods(&self) {
        print_header("methods", self.methods.len());
        for (i, item) in self.methods.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            println!("    impl_signature:     {}", item.impl_signature.inner);
            println!("    ds_structure:       {}", item.ds_structure);
            println!("    function_signature: {}", item.function_signature.inner);
            print_osed_element("method_name", &item.method_name, 2);
            print_osed_element("impl_body",   &item.impl_body,   2);
            print_osed_element("method_body", &item.method_body, 2);
        }
    }

    pub fn print_impls(&self) {
        print_header("impls", self.impls.len());
        for (i, item) in self.impls.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("impl_body", &item.impl_body, 2);
        }
    }

    pub fn print_structs(&self) {
        print_header("structs", self.structs.len());
        for (i, item) in self.structs.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("struct_name", &item.struct_name, 2);
            print_osed_element("struct_body", &item.struct_body, 2);
        }
    }

    pub fn print_traits(&self) {
        print_header("traits", self.traits.len());
        for (i, item) in self.traits.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("trait_name", &item.trait_name, 2);
            print_osed_element("trait_body", &item.trait_body, 2);
        }
    }

    pub fn print_functions(&self) {
        print_header("functions", self.functions.len());
        for (i, item) in self.functions.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("function_name", &item.function_name, 2);
            print_osed_element("function_body", &item.function_body, 2);
        }
    }

    pub fn print_tests_mods(&self) {
        print_header("tests_mods", self.tests_mods.len());
        for (i, item) in self.tests_mods.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("tests_mod_body", &item.tests_mod_body, 2);
        }
    }

    pub fn print_enums(&self) {
        print_header("enums", self.enums.len());
        for (i, item) in self.enums.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("enum_name", &item.enum_name, 2);
            print_osed_element("enum_body", &item.enum_body, 2);
        }
    }

    pub fn print_unions(&self) {
        print_header("unions", self.unions.len());
        for (i, item) in self.unions.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("union_name", &item.union_name, 2);
            print_osed_element("union_body", &item.union_body, 2);
        }
    }

    pub fn print_type_aliases(&self) {
        print_header("type_aliases", self.type_aliases.len());
        for (i, item) in self.type_aliases.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("type_name", &item.type_name, 2);
            print_osed_element("type_body", &item.type_body, 2);
        }
    }

    pub fn print_trait_method_sigs(&self) {
        print_header("trait_method_sigs", self.trait_method_sigs.len());
        for (i, item) in self.trait_method_sigs.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("trait_name",             &item.trait_name,             2);
            print_osed_element("trait_body",             &item.trait_body,             2);
            print_osed_element("method_signature_name",  &item.method_signature_name,  2);
            print_osed_element("trait_method_signature", &item.trait_method_signature, 2);
        }
    }

    pub fn print_trait_method_defs(&self) {
        print_header("trait_method_defs", self.trait_method_defs.len());
        for (i, item) in self.trait_method_defs.iter().enumerate() {
            println!("  [{i}] file: {}", item.file);
            print_osed_element("trait_name",        &item.trait_name,        2);
            print_osed_element("trait_body",        &item.trait_body,        2);
            print_osed_element("method_name",       &item.method_name,       2);
            print_osed_element("trait_method_body", &item.trait_method_body, 2);
        }
    }

    pub fn print_all(&self) {
        self.print_attributes();
        self.print_methods();
        self.print_impls();
        self.print_structs();
        self.print_traits();
        self.print_functions();
        self.print_tests_mods();
        self.print_enums();
        self.print_unions();
        self.print_type_aliases();
        self.print_trait_method_sigs();
        self.print_trait_method_defs();
    }
}
