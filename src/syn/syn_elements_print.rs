//syn_elements_print.rs
use crate::syn::SynElement;
use crate::syn::syn_elements::*;

fn print_header(label: &str, count: usize) {
    println!("\n=== {label} ({count}) ===");
}

fn print_syn_element(label: &str, el: &SynElement, indent: usize) {
    let pad = "  ".repeat(indent);
    let preview = el.text.lines().take(2).collect::<Vec<_>>().join("\n");
    println!("{pad}{label}:");
    println!("{pad}  preview: {:?}", preview);
}

// ── Macro for uniform single-field or multi-field collections ─────────────────

macro_rules! impl_print_syn_collection {
    (
        $method_name:ident => $field:ident {
            $( $el:ident ),+ $(,)?
        }
    ) => {
        pub fn $method_name(&self) {
            print_header(stringify!($field), self.$field.len());
            for (i, item) in self.$field.iter().enumerate() {
                println!("  [{i}] file: {}", item.file);
                $( print_syn_element(stringify!($el), &item.$el, 2); )+
            }
        }
    };
}

// ── AllSynElements print methods ──────────────────────────────────────────────

impl AllSynElements {
    impl_print_syn_collection!(print_attributes => syn_attributes {
        attribute_body,
    });

    impl_print_syn_collection!(print_tests_mods => syn_tests_mods {
        tests_mod_body,
    });

    impl_print_syn_collection!(print_functions => syn_functions {
        function_name,
        function_body,
    });

    impl_print_syn_collection!(print_structs => syn_structs {
        struct_name,
        struct_body,
    });

    impl_print_syn_collection!(print_traits => syn_traits {
        trait_name,
        trait_body,
    });

    impl_print_syn_collection!(print_trait_method_sigs => syn_trait_method_sigs {
        trait_name,
        trait_body,
        method_signature_name,
        trait_method_signature,
    });

    impl_print_syn_collection!(print_trait_method_defs => syn_trait_method_defs {
        trait_name,
        trait_body,
        method_name,
        trait_method_body,
    });

    impl_print_syn_collection!(print_type_aliases => syn_type_aliases {
        type_name,
        type_body,
    });

    impl_print_syn_collection!(print_enums => syn_enums {
        enum_name,
        enum_body,
    });

    impl_print_syn_collection!(print_unions => syn_unions {
        union_name,
        union_body,
    });

    impl_print_syn_collection!(print_expression_statements => syn_expression_statements {
        expression_body,
    });

    impl_print_syn_collection!(print_use_declarations => syn_use_declarations {
        use_body,
    });

    impl_print_syn_collection!(print_macro_definitions => syn_macro_definitions {
        macro_definition_name,
        macro_definition_body,
    });

    impl_print_syn_collection!(print_macro_invocations => syn_macro_invocations {
        invocation_body,
    });

    pub fn print_impls(&self) {
        print_header("syn_impls", self.syn_impls.len());
        for (i, imp) in self.syn_impls.iter().enumerate() {
            println!("  [{i}] file: {}", imp.file);
            print_syn_element("impl_body", &imp.impl_body, 2);
        }
    }

    pub fn print_methods(&self) {
        print_header("syn_methods", self.syn_methods.len());
        for (i, m) in self.syn_methods.iter().enumerate() {
            println!("  [{i}] file: {}", m.file);
            println!("    impl_signature:     {}", m.impl_signature);
            println!("    ds_structure:       {}", m.ds_structure);
            println!("    function_signature: {}", m.function_signature);
            Self::print_type_identifiers(&m.type_identifiers);
            print_syn_element("method_name", &m.method_name, 2);
            print_syn_element("impl_body", &m.impl_body, 2);
            print_syn_element("method_body", &m.method_body, 2);
        }
    }

    pub fn print_mods(&self) {
        print_header("syn_mods", self.syn_mods.len());
        for (i, m) in self.syn_mods.iter().enumerate() {
            println!("  [{i}] file: {}", m.file);
            print_syn_element("mod_name", &m.mod_name, 2);
            print_syn_element("mod_body", &m.mod_body, 2);
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn print_type_identifiers(ti: &TypeIdentifiers) {
        println!("    type_identifiers:");
        match &ti.concrete_types {
            None => println!("      concrete_types: None"),
            Some(ct) => println!("      concrete_types: {:?}", ct),
        }
        match &ti.type_variables {
            None => println!("      type_variables: None"),
            Some(tvars) => {
                println!("      type_variables:");
                let mut keys: Vec<_> = tvars.keys().collect();
                keys.sort();
                for k in keys {
                    println!("        {k}: {:?}", tvars[k]);
                }
            }
        }
    }
}
