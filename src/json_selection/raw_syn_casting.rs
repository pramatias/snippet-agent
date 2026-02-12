use crate::json_selection::raw_elements::*;
use crate::json_selection::unprocessed_elements::*;

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

impl From<MethodSelection> for UnprocessedMethod {
    fn from(s: MethodSelection) -> Self {
        UnprocessedMethod {
            file: s.file,
            impl_body: MethodImplBody {
                text: s.impl_text,
                range: s.impl_range.into(),
            },
            method_body: MethodBody {
                text: s.body_text,
                range: s.body_range.into(),
            },
            method_name: MethodName {
                text: s.name_text,
                range: s.name_range.into(),
            },
        }
    }
}

impl From<TraitMethodDefinitionSelection> for UnprocessedTraitMethodDefinition {
    fn from(s: TraitMethodDefinitionSelection) -> Self {
        UnprocessedTraitMethodDefinition {
            file: s.file,
            trait_body: TraitBody {
                text: s.trait_body_text,
                range: s.trait_body_range.into(),
            },
            trait_method_body: TraitMethodBody {
                text: s.method_body_text,
                range: s.method_body_range.into(),
            },
            method_name: TraitMethodName {
                text: s.method_name_text,
                range: s.method_name_range.into(),
            },
            trait_name: TraitName {
                text: s.trait_name_text,
                range: s.trait_name_range.into(),
            },
        }
    }
}

impl From<TypeAliasSelection> for UnprocessedTypeAlias {
    fn from(s: TypeAliasSelection) -> Self {
        UnprocessedTypeAlias {
            file: s.file,
            type_body: TypeAliasBody {
                text: s.body_text,
                range: s.body_range.into(),
            },
            type_name: TypeAliasName {
                text: s.name_text,
                range: s.name_range.into(),
            },
        }
    }
}

impl From<EnumSelection> for UnprocessedEnum {
    fn from(s: EnumSelection) -> Self {
        UnprocessedEnum {
            file: s.file,
            enum_body: EnumBody {
                text: s.enum_body_text,
                range: s.enum_body_range.into(),
            },
            enum_name: EnumName {
                text: s.enum_name_text,
                range: s.enum_name_range.into(),
            },
        }
    }
}

impl From<UnionSelection> for UnprocessedUnion {
    fn from(s: UnionSelection) -> Self {
        UnprocessedUnion {
            file: s.file,
            union_body: UnionBody {
                text: s.union_body_text,
                range: s.union_body_range.into(),
            },
            union_name: UnionName {
                text: s.union_name_text,
                range: s.union_name_range.into(),
            },
        }
    }
}

impl From<TestsModSelection> for UnprocessedTestsMod {
    fn from(s: TestsModSelection) -> Self {
        UnprocessedTestsMod {
            file: s.file,
            tests_mod_body: TestsModBody {
                text: s.tests_mod,
                range: s.range.into(),
            },
        }
    }
}

impl From<FunctionSelection> for UnprocessedFunction {
    fn from(s: FunctionSelection) -> Self {
        UnprocessedFunction {
            file: s.file,
            function_body: FunctionBody {
                text: s.body_text,
                range: s.body_range.into(),
            },
            function_name: FunctionName {
                text: s.name_text,
                range: s.name_range.into(),
            },
        }
    }
}

impl From<TraitSelection> for UnprocessedTrait {
    fn from(s: TraitSelection) -> Self {
        UnprocessedTrait {
            file: s.file,
            trait_body: TraitBody {
                text: s.trait_body_text,
                range: s.trait_body_range.into(),
            },
            trait_name: TraitName {
                text: s.trait_name_text,
                range: s.trait_name_range.into(),
            },
        }
    }
}

impl From<TraitMethodSignatureSelection> for UnprocessedTraitMethodSignature {
    fn from(s: TraitMethodSignatureSelection) -> Self {
        UnprocessedTraitMethodSignature {
            file: s.file,
            trait_method_signature: TraitMethodSignature {
                text: s.signature_text,
                range: s.signature_range.into(),
            },
            method_signature_name: SignatureName {
                text: s.signature_name_text,
                range: s.signature_name_range.into(),
            },
            trait_body: TraitBody {
                text: s.enclosing_trait_text,
                range: s.enclosing_trait_range.into(),
            },
            trait_name: TraitName {
                text: s.trait_name_text,
                range: s.trait_name_range.into(),
            },
        }
    }
}

impl From<AttributeSelection> for UnprocessedAttribute {
    fn from(s: AttributeSelection) -> Self {
        UnprocessedAttribute {
            file: s.file,
            attribute_body: AttributeBody {
                text: s.attribute,
                range: s.range.into(),
            },
        }
    }
}

impl From<ImplSelection> for UnprocessedImpl {
    fn from(s: ImplSelection) -> Self {
        UnprocessedImpl {
            file: s.file,
            impl_body: ImplBody {
                text: s.impl_text,
                range: s.impl_range.into(),
            },
        }
    }
}

impl From<StructSelection> for UnprocessedStruct {
    fn from(s: StructSelection) -> Self {
        UnprocessedStruct {
            file: s.file,
            struct_body: StructBody {
                text: s.body_text,
                range: s.body_range.into(),
            },
            struct_name: StructName {
                text: s.name_text,
                range: s.name_range.into(),
            },
        }
    }
}
