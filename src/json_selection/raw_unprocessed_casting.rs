use crate::json_selection::raw_elements::*;
use crate::json_selection::unprocessed_elements::*;
use syntax_queries::{ByteRange, CharactersDimension, SynPosition, SynRange};

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
    (
        $source_type:ty => $target_type:ty {
            $(
                $target_field:ident: $wrapper_type:tt { $source_text:ident, $source_range:ident }
            ),+ $(,)?
        }
    ) => {
        impl From<$source_type> for $target_type {
            fn from(s: $source_type) -> Self {
                Self {
                    file: s.file,
                    $(
                        $target_field: $wrapper_type {
                            text: s.$source_text,
                            range: s.$source_range.into(),
                        },
                    )+
                }
            }
        }
    };
}

impl_from_selection!(MethodSelection => UnprocessedMethod {
    impl_body: MethodImplBody { impl_text, impl_range },
    method_body: MethodBody { body_text, body_range },
    method_name: MethodName { name_text, name_range },
});

impl_from_selection!(TraitMethodDefinitionSelection => UnprocessedTraitMethodDefinition {
    trait_body: TraitBody { trait_body_text, trait_body_range },
    trait_method_body: TraitMethodBody { method_body_text, method_body_range },
    method_name: TraitMethodName { method_name_text, method_name_range },
    trait_name: TraitName { trait_name_text, trait_name_range },
});

impl_from_selection!(TypeAliasSelection => UnprocessedTypeAlias {
    type_body: TypeAliasBody { body_text, body_range },
    type_name: TypeAliasName { name_text, name_range },
});

impl_from_selection!(EnumSelection => UnprocessedEnum {
    enum_body: EnumBody { enum_body_text, enum_body_range },
    enum_name: EnumName { enum_name_text, enum_name_range },
});

impl_from_selection!(UnionSelection => UnprocessedUnion {
    union_body: UnionBody { union_body_text, union_body_range },
    union_name: UnionName { union_name_text, union_name_range },
});

impl_from_selection!(TestsModSelection => UnprocessedTestsMod {
    tests_mod_body: TestsModBody { tests_mod, range },
});

impl_from_selection!(FunctionSelection => UnprocessedFunction {
    function_body: FunctionBody { body_text, body_range },
    function_name: FunctionName { name_text, name_range },
});

impl_from_selection!(TraitSelection => UnprocessedTrait {
    trait_body: TraitBody { trait_body_text, trait_body_range },
    trait_name: TraitName { trait_name_text, trait_name_range },
});

impl_from_selection!(TraitMethodSignatureSelection => UnprocessedTraitMethodSignature {
    trait_method_signature: TraitMethodSignature { signature_text, signature_range },
    method_signature_name: SignatureName { signature_name_text, signature_name_range },
    trait_body: TraitBody { enclosing_trait_text, enclosing_trait_range },
    trait_name: TraitName { trait_name_text, trait_name_range },
});

impl_from_selection!(AttributeSelection => UnprocessedAttribute {
    attribute_body: AttributeBody { attribute, range },
});

impl_from_selection!(ImplSelection => UnprocessedImpl {
    impl_body: ImplBody { impl_text, impl_range },
});

impl_from_selection!(StructSelection => UnprocessedStruct {
    struct_body: StructBody { body_text, body_range },
    struct_name: StructName { name_text, name_range },
});
