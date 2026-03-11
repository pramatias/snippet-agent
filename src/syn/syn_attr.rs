//syn_attr.rs
use crate::syn::file_syn_elements::AnyFileSynElement;
use crate::syn::file_syn_elements::FileSynElements;
use crate::syn::syn_elements::SynAttribute;
use syntax_queries::byte_range_ordering::HasByteRange;

///merge_attrs
impl SynAttribute {
    pub fn merge_attrs(attrs: Vec<SynAttribute>, fse: &FileSynElements) -> Vec<SynAttribute> {
        let mut sorted = attrs;
        sorted.sort_by_key(|a| a.attribute_body.byte_range().start);
        let mut result: Vec<SynAttribute> = Vec::new();
        for current in sorted {
            let can_merge = result.last().map_or(false, |prev: &SynAttribute| {
                let immediately_before = matches!(
                    <AnyFileSynElement as HasByteRange>::immediate_before(
                        &fse.elements,
                        &current.attribute_body,
                    ),
                    Some(AnyFileSynElement::Attribute(_))
                );
                if !immediately_before {
                    return false;
                }
                !fse.elements.iter().any(|el| {
                    matches!(
                        el,
                        AnyFileSynElement::TestsMod(_)
                            | AnyFileSynElement::Impl(_)
                            | AnyFileSynElement::Trait(_)
                            | AnyFileSynElement::Function(_)
                    ) && el.byte_range().contains(current.attribute_body.byte_range())
                        && !el.byte_range().contains(prev.attribute_body.byte_range())
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

///merge_with
impl SynAttribute {
    pub fn merge_with(&self, other: &SynAttribute, context: &str) -> SynAttribute {
        SynAttribute {
            file: self.file.clone(),
            attribute_body: self.attribute_body.merge(&other.attribute_body),
            context_lines: context.to_string(),
        }
    }
}
