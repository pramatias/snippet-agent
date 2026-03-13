//unprocessed_syn_casting.rs
use crate::json_selection::raw_elements::*;
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_element::*;
use crate::syn::syn_elements::*;
use std::sync::Arc;

use std::collections::HashMap;

///from_unprocessed
impl AllSynElements {
    /// Build an `AllSynElements` from the raw deserialized layer.
    ///
    /// Pass-through collections (whose `Syn*` aliases ARE `Vec<Arc<Unprocessed*>>`)
    /// are just wrapped in `Arc`.  Enriched structs (`SynMethod`, `SynAttribute`,
    /// etc.) are constructed field-by-field; fields that require a later
    /// enrichment pass (signatures, `ds_structure`, `type_identifiers`) are
    /// left at their `Default`.
    ///
    /// `file_contents` is not consumed here — impl text was captured directly
    /// from the AST grep output into `SynElementWithText` and needs no
    /// file-contents lookup at this stage. It is retained as a parameter
    /// because the enrichment pass that fills `impl_signature`,
    /// `function_signature`, and `type_identifiers` will need it.
    pub fn from_unprocessed(
        all: AllUnprocessedElements,
        file_contents: &HashMap<FilePath, String>,
    ) -> Self {
        let mut out = AllSynElements::default();

        // ── Pass-through: Syn* == Vec<Arc<Unprocessed*>> ─────────────────────
        out.syn_tests_mods = all
            .unprocessed_tests_mods
            .into_iter()
            .map(Arc::new)
            .collect();
        out.syn_functions = all
            .unprocessed_functions
            .into_iter()
            .map(Arc::new)
            .collect();
        out.syn_structs = all.unprocessed_structs.into_iter().map(Arc::new).collect();
        out.syn_traits = all.unprocessed_traits.into_iter().map(Arc::new).collect();
        out.syn_trait_method_sigs = all
            .unprocessed_trait_method_sigs
            .into_iter()
            .map(Arc::new)
            .collect();
        out.syn_trait_method_defs = all
            .unprocessed_trait_method_defs
            .into_iter()
            .map(Arc::new)
            .collect();
        out.syn_type_aliases = all
            .unprocessed_type_aliases
            .into_iter()
            .map(Arc::new)
            .collect();
        out.syn_enums = all.unprocessed_enums.into_iter().map(Arc::new).collect();
        out.syn_unions = all.unprocessed_unions.into_iter().map(Arc::new).collect();
        // impl_body is SynElementWithText — text was captured at extraction
        // time and moves through here without a file-contents lookup.
        out.syn_impls = all.unprocessed_impls.into_iter().map(Arc::new).collect();

        // ── Enriched: build from unprocessed fields ───────────────────────────
        out.syn_attributes = all
            .unprocessed_attributes
            .into_iter()
            .map(|a| {
                Arc::new(SynAttribute {
                    file: a.file,
                    attribute_body: a.attribute_body,
                })
            })
            .collect();

        // impl_body is SynElementWithText and moves directly into SynMethod.
        // Enrichment fields (impl_signature, function_signature, ds_structure,
        // type_identifiers) are unknown at this stage; filled by a later pass.
        out.syn_methods = all
    .unprocessed_methods
    .into_iter()
    .map(|m| {
        let file_content = file_contents
            .get(&m.file)
            .map(String::as_str)
            .unwrap_or("");
        Arc::new(SynMethod::from_unprocessed(m, file_content))
    })
    .collect();

        out.syn_mods = all
            .unprocessed_mods
            .into_iter()
            .map(|m| {
                Arc::new(SynMod {
                    file: m.file,
                    mod_name: m.mod_name,
                    mod_body: m.mod_body,
                })
            })
            .collect();

        out.syn_expression_statements = all
            .unprocessed_expression_statements
            .into_iter()
            .map(|e| {
                Arc::new(SynExpressionStatement {
                    file: e.file,
                    expression_body: e.expression_body,
                })
            })
            .collect();

        out.syn_use_declarations = all
            .unprocessed_use_declarations
            .into_iter()
            .map(|u| {
                Arc::new(SynUseDeclaration {
                    file: u.file,
                    use_body: u.use_body,
                })
            })
            .collect();

        out.syn_macro_definitions = all
            .unprocessed_macro_definitions
            .into_iter()
            .map(|m| {
                Arc::new(SynMacroDefinition {
                    file: m.file,
                    macro_definition_name: m.macro_name,
                    macro_definition_body: m.macro_body,
                })
            })
            .collect();

        out.syn_macro_invocations = all
            .unprocessed_macro_invocations
            .into_iter()
            .map(|m| {
                Arc::new(SynMacroInvocation {
                    file: m.file,
                    invocation_body: m.invocation_body,
                })
            })
            .collect();

        out
    }
}

macro_rules! impl_from_selection {
    ($from:ty => $to:ty {
        $( $field:ident : $wrapper:ident { $source_range:ident } ),+ $(,)?
    }) => {
        impl From<$from> for $to {
            fn from(s: $from) -> Self {
                Self {
                    file: s.file.into(),
                    $(
                        $field: $wrapper::new(s.$source_range),
                    )+
                }
            }
        }
    };
}

impl_from_selection!(TraitMethodDefinitionSelection => UnprocessedTraitMethodDefinition {
    trait_body:        TraitBody       { trait_body_range   },
    trait_method_body: TraitMethodBody { method_body_range  },
    method_name:       TraitMethodName { method_name_range  },
    trait_name:        TraitName       { trait_name_range   },
});
impl_from_selection!(TypeAliasSelection => UnprocessedTypeAlias {
    type_body: TypeAliasBody { body_range },
    type_name: TypeAliasName { name_range },
});
impl_from_selection!(EnumSelection => UnprocessedEnum {
    enum_body: EnumBody { enum_body_range },
    enum_name: EnumName { enum_name_range },
});
impl_from_selection!(UnionSelection => UnprocessedUnion {
    union_body: UnionBody { union_body_range },
    union_name: UnionName { union_name_range },
});
impl_from_selection!(TestsModSelection => UnprocessedTestsMod {
    tests_mod_body: TestsModBody { range },
});
impl_from_selection!(FunctionSelection => UnprocessedFunction {
    function_body: FunctionBody { body_range },
    function_name: FunctionName { name_range },
});
impl_from_selection!(TraitSelection => UnprocessedTrait {
    trait_body: TraitBody { trait_body_range },
    trait_name: TraitName { trait_name_range },
});
impl_from_selection!(TraitMethodSignatureSelection => UnprocessedTraitMethodSignature {
    trait_method_signature: TraitMethodSignature { signature_range       },
    method_signature_name:  SignatureName        { signature_name_range  },
    trait_body:             TraitBody            { enclosing_trait_range },
    trait_name:             TraitName            { trait_name_range      },
});
impl_from_selection!(AttributeSelection => UnprocessedAttribute {
    attribute_body: AttributeBody { range },
});
impl_from_selection!(StructSelection => UnprocessedStruct {
    struct_body: StructBody { body_range },
    struct_name: StructName { name_range },
});
impl_from_selection!(ModSelection => UnprocessedMod {
    mod_body: ModBody { mod_body_range },
    mod_name: ModName { mod_name_range },
});
impl_from_selection!(ExpressionStatementSelection => UnprocessedExpressionStatement {
    expression_body: ExpressionStatementBody { expression_range },
});
impl_from_selection!(UseDeclarationSelection => UnprocessedUseDeclaration {
    use_body: UseDeclarationBody { use_range },
});
impl_from_selection!(MacroDefinitionSelection => UnprocessedMacroDefinition {
    macro_body: MacroDefinitionBody { macro_body_range },
    macro_name: MacroDefinitionName { macro_name_range },
});
impl_from_selection!(MacroInvocationSelection => UnprocessedMacroInvocation {
    invocation_body: MacroInvocationBody { invocation_range },
});

impl From<MethodSelection> for UnprocessedMethod {
    fn from(s: MethodSelection) -> Self {
        Self {
            file: s.file.into(),
            impl_body: SynElementWithText::new(s.impl_range.into(), s.impl_text),
            method_body: MethodBody::new(s.body_range),
            method_name: MethodName::new(s.name_range),
        }
    }
}

impl From<ImplSelection> for UnprocessedImpl {
    fn from(s: ImplSelection) -> Self {
        Self {
            file: s.file.into(),
            impl_body: SynElementWithText::new(s.impl_range.into(), s.impl_text),
        }
    }
}
