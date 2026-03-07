pub trait HasByteRange {
    fn byte_range(&self) -> &ByteRange;

    fn before(&self, other: &impl HasByteRange) -> bool {
        self.byte_range().before(other.byte_range())
    }
    fn after(&self, other: &impl HasByteRange) -> bool {
        self.byte_range().after(other.byte_range())
    }
    fn contains(&self, other: &impl HasByteRange) -> bool {
        self.byte_range().contains(other.byte_range())
    }

    fn before_all<'a, T: HasByteRange>(items: &'a [T], divider: &impl HasByteRange) -> Vec<&'a T> {
        filter_range_all!(before, items, divider)
    }
    fn after_all<'a, T: HasByteRange>(items: &'a [T], divider: &impl HasByteRange) -> Vec<&'a T> {
        filter_range_all!(after, items, divider)
    }

    fn immediate_before<'a, T: HasByteRange>(
        items: &'a [T],
        limit: &impl HasByteRange,
    ) -> Option<&'a T> {
        filter_range_immediate!(before, max_by_key, end, items, limit)
    }
    fn immediate_after<'a, T: HasByteRange>(
        items: &'a [T],
        limit: &impl HasByteRange,
    ) -> Option<&'a T> {
        filter_range_immediate!(after, min_by_key, start, items, limit)
    }

    fn first_contained<'a, T: HasByteRange>(
        items: &'a [T],
        container: &impl HasByteRange,
    ) -> Option<&'a T> {
        filter_range_contained!(min_by_key, items, container)
    }

    fn last_contained<'a, T: HasByteRange>(
        items: &'a [T],
        container: &impl HasByteRange,
    ) -> Option<&'a T> {
        filter_range_contained!(max_by_key, items, container)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct SynPosition {
    pub line: u64,
    pub column: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct CharactersDimension {
    pub start: SynPosition,
    pub end: SynPosition,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct SynRange {
    pub byte_range: ByteRange,
    pub characters_dimension: CharactersDimension,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct NodeMatch {
    pub text: String,

    #[allow(dead_code)]
    pub range: SynRange,
}

impl ByteRange {
    /// Self ends before other starts (no overlap)
    pub fn before(&self, other: &ByteRange) -> bool {
        self.end <= other.start
    }

    /// Self starts after other ends (no overlap)
    pub fn after(&self, other: &ByteRange) -> bool {
        self.start >= other.end
    }

    /// Self fully contains other
    pub fn contains(&self, other: &ByteRange) -> bool {
        self.start <= other.start && self.end >= other.end
    }
}

impl HasByteRange for ByteRange {
    fn byte_range(&self) -> &ByteRange {
        self
    }
}

impl HasByteRange for NodeMatch {
    fn byte_range(&self) -> &ByteRange {
        &self.range.byte_range
    }
}

impl HasByteRange for &NodeMatch {
    fn byte_range(&self) -> &ByteRange {
        &self.range.byte_range
    }
}

impl SynRange {
    /// Merge two ranges so that the result spans from the earlier start to the
    /// later end.  The `characters_dimension` is merged the same way.
    pub fn merge(&self, other: &SynRange) -> SynRange {
        SynRange {
            byte_range: self.byte_range.merge(&other.byte_range),
            characters_dimension: self
                .characters_dimension
                .merge(&other.characters_dimension),
        }
    }
}

impl ByteRange {
    /// Produce a range that spans both inputs.
    pub fn merge(&self, other: &ByteRange) -> ByteRange {
        ByteRange {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl CharactersDimension {
    /// Produce a dimension that spans both inputs (earliest start, latest end).
    pub fn merge(&self, other: &CharactersDimension) -> CharactersDimension {
        // "earlier" start = smaller (line, column) pair
        let start = if (self.start.line, self.start.column)
            <= (other.start.line, other.start.column)
        {
            self.start.clone()
        } else {
            other.start.clone()
        };

        // "later" end = larger (line, column) pair
        let end =
            if (self.end.line, self.end.column) >= (other.end.line, other.end.column) {
                self.end.clone()
            } else {
                other.end.clone()
            };

        CharactersDimension { start, end }
    }
}
