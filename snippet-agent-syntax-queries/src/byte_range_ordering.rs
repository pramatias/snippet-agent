//byte_range_ordering.rs
use serde::Deserialize;

// All slice methods assume the slice is sorted by byte_range().start ascending,
// ties broken by end descending (outermost first).  That invariant is
// established by FileSynElements::from_all_syn_elements and must be preserved
// by any code that mutates FileSynElements::elements.

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

    /// All items whose range ends at or before `divider` starts.
    fn before_all<'a, T: HasByteRange>(items: &'a [T], divider: &impl HasByteRange) -> Vec<&'a T> {
        let divider_start = divider.byte_range().start;
        // Items with start >= divider_start have end >= start >= divider_start,
        // so they can never satisfy `end <= divider_start`.
        let hi = items.partition_point(|x| x.byte_range().start < divider_start);
        items[..hi]
            .iter()
            .filter(|x| x.byte_range().end <= divider_start)
            .collect()
    }

    /// All items whose range starts at or after `divider` ends.
    fn after_all<'a, T: HasByteRange>(items: &'a [T], divider: &impl HasByteRange) -> Vec<&'a T> {
        let divider_end = divider.byte_range().end;
        let lo = items.partition_point(|x| x.byte_range().start < divider_end);
        items[lo..].iter().collect()
    }

    /// The item immediately before `limit` (ends earliest while still before `limit`).
    fn immediate_before<'a, T: HasByteRange>(
        items: &'a [T],
        limit: &impl HasByteRange,
    ) -> Option<&'a T> {
        let limit_start = limit.byte_range().start;
        let hi = items.partition_point(|x| x.byte_range().start < limit_start);
        items[..hi]
            .iter()
            .filter(|x| x.byte_range().end <= limit_start)
            .max_by_key(|x| x.byte_range().end)
    }

    /// The item immediately after `limit` (starts latest while still after `limit`).
    fn immediate_after<'a, T: HasByteRange>(
        items: &'a [T],
        limit: &impl HasByteRange,
    ) -> Option<&'a T> {
        let limit_end = limit.byte_range().end;
        let lo = items.partition_point(|x| x.byte_range().start < limit_end);
        // Slice is sorted by start, so the first element at `lo` has the minimum start.
        items.get(lo)
    }

    /// The first (earliest-start) item fully contained within `container`.
    fn first_contained<'a, T: HasByteRange>(
        items: &'a [T],
        container: &impl HasByteRange,
    ) -> Option<&'a T> {
        let range = container.byte_range();
        let lo = items.partition_point(|x| x.byte_range().start < range.start);
        items[lo..]
            .iter()
            // Once start exceeds container.end nothing further can be contained.
            .take_while(|x| x.byte_range().start <= range.end)
            .find(|x| x.byte_range().end <= range.end)
    }

    /// The last (latest-start) item fully contained within `container`.
    fn last_contained<'a, T: HasByteRange>(
        items: &'a [T],
        container: &impl HasByteRange,
    ) -> Option<&'a T> {
        let range = container.byte_range();
        let lo = items.partition_point(|x| x.byte_range().start < range.start);
        let hi = items.partition_point(|x| x.byte_range().start <= range.end);
        items[lo..hi]
            .iter()
            .filter(|x| x.byte_range().end <= range.end)
            .last()
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
            characters_dimension: self.characters_dimension.merge(&other.characters_dimension),
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
        let start =
            if (self.start.line, self.start.column) <= (other.start.line, other.start.column) {
                self.start.clone()
            } else {
                other.start.clone()
            };

        // "later" end = larger (line, column) pair
        let end = if (self.end.line, self.end.column) >= (other.end.line, other.end.column) {
            self.end.clone()
        } else {
            other.end.clone()
        };

        CharactersDimension { start, end }
    }
}

impl Default for SynPosition {
    fn default() -> Self {
        SynPosition { line: 0, column: 0 }
    }
}

impl Default for ByteRange {
    fn default() -> Self {
        ByteRange { start: 0, end: 0 }
    }
}

impl Default for CharactersDimension {
    fn default() -> Self {
        CharactersDimension {
            start: SynPosition::default(),
            end: SynPosition::default(),
        }
    }
}

impl Default for SynRange {
    fn default() -> Self {
        SynRange {
            byte_range: ByteRange::default(),
            characters_dimension: CharactersDimension::default(),
        }
    }
}

impl Default for NodeMatch {
    fn default() -> Self {
        NodeMatch {
            text: String::new(),
            range: SynRange::default(),
        }
    }
}
