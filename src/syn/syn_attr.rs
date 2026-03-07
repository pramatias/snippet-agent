
///merge_attrs
impl SynAttribute {
    pub fn merge_attrs(
        attrs: Vec<SynAttribute>,
        fse: &FileSynElements,
    ) -> Vec<SynAttribute> {
        let mut sorted = attrs;
        sorted.sort_by_key(|a| a.byte_range().start);
        let mut result: Vec<SynAttribute> = Vec::new();
        for current in sorted {
            let can_merge = result.last().map_or(false, |prev: &SynAttribute| {
                let immediately_before = matches!(
                    <AnyFileSynElement as HasByteRange>::immediate_before(&fse.elements, &current),
                    Some(AnyFileSynElement::Attribute(_))
                );
                if !immediately_before {
                    return false;
                }
                // Only block if a *scope-creating* element contains `current`
                // but not `prev`. Struct/union/enum bodies are excluded because
                // their byte ranges can start mid-attribute-list.
                !fse.elements.iter().any(|el| {
                    matches!(
                        el,
                        AnyFileSynElement::TestsMod(_)
                            | AnyFileSynElement::Impl(_)
                            | AnyFileSynElement::Trait(_)
                            | AnyFileSynElement::Function(_)
                    ) && el.byte_range().contains(current.byte_range())
                        && !el.byte_range().contains(prev.byte_range())
                })
            });
            if can_merge {
                let prev = result.pop().unwrap();
                let merged_body = prev.attribute_body.merge(&current.attribute_body);
                result.push(SynAttribute {
                    file: prev.file,
                    attribute_body: merged_body,
                    context_lines: String::new(),
                });
            } else {
                result.push(current);
            }
        }
        result
    }
}
