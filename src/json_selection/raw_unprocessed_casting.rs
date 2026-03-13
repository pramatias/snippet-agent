// raw_unprocessed_casting.rs
use crate::json_selection::raw_elements::*;
use crate::json_selection::unprocessed_elements::*;
use crate::syn::*;
use std::sync::Arc;
use syntax_queries::byte_range_ordering::{ByteRange, CharactersDimension, SynPosition, SynRange};

impl From<ByteOffset> for ByteRange {
    fn from(b: ByteOffset) -> Self {
        ByteRange {
            start: b.start,
            end: b.end,
        }
    }
}

impl From<Position> for SynPosition {
    fn from(p: Position) -> Self {
        SynPosition {
            line: p.line,
            column: p.column,
        }
    }
}

impl From<Range> for SynRange {
    fn from(r: Range) -> Self {
        SynRange {
            byte_range: r.byte_offset.into(),
            characters_dimension: CharactersDimension {
                start: r.start.into(),
                end: r.end.into(),
            },
        }
    }
}

macro_rules! impl_from_selection {
    ($from:ty => $to:ty {
        $( $field:ident : $wrapper:ident { $source_range:ident } ),+ $(,)?
    }) => {
        impl $to {
            pub fn from_selection(s: $from) -> Self {
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

// impl From<MethodSelection> for UnprocessedMethod {
//     fn from(s: MethodSelection) -> Self {
//         Self {
//             file:        s.file.into(),
//             impl_body:   MethodImplBody::new(s.impl_range.into(), s.impl_text),
//             method_body: MethodBody::new(s.body_range),
//             method_name: MethodName::new(s.name_range),
//         }
//     }
// }

// impl From<ImplSelection> for UnprocessedImpl {
//     fn from(s: ImplSelection) -> Self {
//         Self {
//             file:      s.file.into(),
//             impl_body: ImplBody::new(s.impl_range.into(), s.impl_text),
//         }
//     }
// }
