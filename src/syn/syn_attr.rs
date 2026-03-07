impl SynAttribute {
    /// Merge adjacent attributes in `attrs`, using `fse` to check that no
    /// other SynElement sits between two consecutive attributes.
    ///
    /// Two attributes A and B merge iff:
    ///   - B comes after A (by byte range)
    ///   - The element immediately before B in `fse` is A itself
    ///
    /// The merged result is written back into `all.syn_attributes`.
    pub fn merge_attrs(
        attrs: Vec<SynAttribute>,
        fse: &FileSynElements,
    ) -> Vec<SynAttribute> {
        // Sort by start byte
        let mut sorted = attrs;
        sorted.sort_by_key(|a| a.byte_range().start);

        let mut result: Vec<SynAttribute> = Vec::new();

        for current in sorted {
            // Check if the element immediately before `current` in fse
            // is the last accumulated attribute (i.e. nothing sits between them)
            let can_merge = result.last().map_or(false, |prev: &SynAttribute| {
                match <AnyFileSynElement as HasByteRange>::immediate_before(&fse.elements, &current) {
                    Some(AnyFileSynElement::Attribute(before_attr)) => {
                        before_attr.byte_range() == prev.byte_range()
                    }
                    _ => false,
                }
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
