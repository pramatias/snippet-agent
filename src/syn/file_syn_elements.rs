// file_syn_elements.rs
use crate::AllSynElements;
use crate::json_selection::unprocessed_elements::*;
use crate::syn::FilePath;
use crate::syn::syn_element::SynElement;
use crate::syn::syn_elements::{SynAttribute, SynMethod};
use rayon::prelude::*;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use syntax_queries::byte_range_ordering::{ByteRange, HasByteRange};

// ── AnyFileSynElement ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AnyFileSynElement {
    Sentinel,
    Attribute(Arc<SynAttribute>),
    Method(Arc<SynMethod>),
    Impl(Arc<UnprocessedImpl>),
    Struct(Arc<UnprocessedStruct>),
    Trait(Arc<UnprocessedTrait>),
    Function(Arc<UnprocessedFunction>),
    TestsMod(Arc<UnprocessedTestsMod>),
    Enum(Arc<UnprocessedEnum>),
    Union(Arc<UnprocessedUnion>),
    TypeAlias(Arc<UnprocessedTypeAlias>),
    TraitMethodSig(Arc<UnprocessedTraitMethodSignature>),
    TraitMethodDef(Arc<UnprocessedTraitMethodDefinition>),
}

impl HasByteRange for AnyFileSynElement {
    fn byte_range(&self) -> &ByteRange {
        match self {
            Self::Attribute(x) => x.attribute_body.byte_range(),
            Self::Method(x) => x.impl_body.byte_range(),
            Self::Impl(x) => x.impl_body.byte_range(),
            Self::Struct(x) => x.struct_body.byte_range(),
            Self::Trait(x) => x.trait_body.byte_range(),
            Self::Function(x) => x.function_body.byte_range(),
            Self::TestsMod(x) => x.tests_mod_body.byte_range(),
            Self::Enum(x) => x.enum_body.byte_range(),
            Self::Union(x) => x.union_body.byte_range(),
            Self::TypeAlias(x) => x.type_body.byte_range(),
            Self::TraitMethodSig(x) => x.trait_method_signature.byte_range(),
            Self::TraitMethodDef(x) => x.trait_method_body.byte_range(),
            Self::Sentinel => panic!("byte_range called on sentinel root"),
        }
    }
}

impl AnyFileSynElement {
    pub fn is_attribute(&self) -> bool {
        matches!(self, Self::Attribute(_))
    }
}

// ── FileSynElements ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileSynElements {
    pub file: FilePath,
    /// Sorted by `byte_range().start` ascending; ties broken by end descending
    /// (outermost node first).
    pub elements: Vec<AnyFileSynElement>,
}

// ── FileSynElementsMap ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileSynElementsMap(pub BTreeMap<FilePath, FileSynElements>);

impl FileSynElementsMap {
    /// Borrows `all` — every Arc clone is O(1), so this is equivalent in cost
    /// to the old consuming version but leaves `all` intact for the retain pass.
    pub fn from_all_syn_elements(all: &AllSynElements) -> Self {
        Self(FileSynElements::from_all_syn_elements(all))
    }

    /// Filter every file's element list in parallel.
    pub fn filter_by_tree(&mut self, max_depth: usize, max_attrs: usize) {
        let fses: Vec<&mut FileSynElements> = self.0.values_mut().collect();
        fses.into_par_iter().for_each(|fse| {
            crate::syn::file_syn_elements_tree::filter_elements(fse, max_depth, max_attrs);
        });
    }

    /// Collect surviving `(start, end)` byte-range pairs keyed by file path.
    ///
    /// Using both start *and* end avoids false positives when two elements
    /// share a start byte (e.g. an attribute immediately before its target).
    pub fn surviving_ranges(&self) -> HashMap<FilePath, HashSet<(u64, u64)>> {
        self.0
            .iter()
            .map(|(file, fse)| {
                let ranges = fse
                    .elements
                    .iter()
                    .map(|e| {
                        let r = e.byte_range();
                        (r.start, r.end)
                    })
                    .collect::<HashSet<_>>();
                (Arc::clone(file), ranges) // Arc::clone is O(1)
            })
            .collect()
    }
}

// ── FileSynElements::from_all_syn_elements ────────────────────────────────────

impl FileSynElements {
    pub fn from_all_syn_elements(all: &AllSynElements) -> BTreeMap<FilePath, FileSynElements> {
        // ── Pass 1: count elements per file so Vecs are pre-sized ────────────
        let mut counts: BTreeMap<FilePath, usize> = BTreeMap::new();
        macro_rules! count_all {
            ($vec:expr) => {
                for el in &$vec {
                    *counts.entry(el.file.clone()).or_insert(0) += 1;
                }
            };
        }
        count_all!(all.syn_attributes);
        count_all!(all.syn_methods);
        count_all!(all.syn_impls);
        count_all!(all.syn_structs);
        count_all!(all.syn_traits);
        count_all!(all.syn_functions);
        count_all!(all.syn_tests_mods);
        count_all!(all.syn_enums);
        count_all!(all.syn_unions);
        count_all!(all.syn_type_aliases);
        count_all!(all.syn_trait_method_sigs);
        count_all!(all.syn_trait_method_defs);

        // ── Pass 2: build map with pre-sized Vecs ────────────────────────────
        let mut map: BTreeMap<FilePath, FileSynElements> = counts
            .into_iter()
            .map(|(file, cap)| {
                (
                    file.clone(),
                    FileSynElements {
                        file,
                        elements: Vec::with_capacity(cap),
                    },
                )
            })
            .collect();

        // Arc::clone is O(1); all keys were seeded above so unwrap() is safe.
        macro_rules! insert_all {
            ($vec:expr, $variant:expr) => {
                for el in &$vec {
                    map.get_mut(&el.file as &str)
                        .unwrap()
                        .elements
                        .push($variant(Arc::clone(el)));
                }
            };
        }
        insert_all!(all.syn_attributes, AnyFileSynElement::Attribute);
        insert_all!(all.syn_methods, AnyFileSynElement::Method);
        insert_all!(all.syn_impls, AnyFileSynElement::Impl);
        insert_all!(all.syn_structs, AnyFileSynElement::Struct);
        insert_all!(all.syn_traits, AnyFileSynElement::Trait);
        insert_all!(all.syn_functions, AnyFileSynElement::Function);
        insert_all!(all.syn_tests_mods, AnyFileSynElement::TestsMod);
        insert_all!(all.syn_enums, AnyFileSynElement::Enum);
        insert_all!(all.syn_unions, AnyFileSynElement::Union);
        insert_all!(all.syn_type_aliases, AnyFileSynElement::TypeAlias);
        insert_all!(all.syn_trait_method_sigs, AnyFileSynElement::TraitMethodSig);
        insert_all!(all.syn_trait_method_defs, AnyFileSynElement::TraitMethodDef);

        // ── Pass 3: sort each file's elements once ────────────────────────────
        // start asc, end desc (outermost/enclosing node comes first on ties)
        for fse in map.values_mut() {
            fse.elements.sort_by(|a, b| {
                let ra = a.byte_range();
                let rb = b.byte_range();
                ra.start.cmp(&rb.start).then(rb.end.cmp(&ra.end))
            });
        }

        map
    }
}

// ── AllSynElements helpers ────────────────────────────────────────────────────

impl AllSynElements {
    /// Filter in place, avoiding the full `AllSynElements → map → AllSynElements`
    /// round-trip allocation.
    ///
    /// Memory profile:
    ///   peak  = `self` (1×) + map duplicate Arcs (1×, dropped before retain)
    ///   old   = `self` (1×) + map (1×) + `all_filtered` (1×) — three live at once
    ///
    /// The `retain()` calls are in-place shifts on the existing Vec backing
    /// arrays — no per-element allocation.
    pub fn filter_by_tree_in_place(&mut self, max_depth: usize, max_attrs: usize) {
        let mut map = FileSynElementsMap::from_all_syn_elements(self);
        map.filter_by_tree(max_depth, max_attrs);
        let survivors = map.surviving_ranges();
        // Drop duplicate Arcs before the retain pass so peak memory is 2× not 3×.
        drop(map);

        macro_rules! retain_all {
            ($vec:expr, $body:ident) => {
                $vec.retain(|el| {
                    let r = el.$body.byte_range();
                    survivors
                        .get(&el.file) // &Arc<str> lookup, no as_ref() needed
                        .map_or(false, |s| s.contains(&(r.start, r.end)))
                });
            };
        }
        retain_all!(self.syn_attributes, attribute_body);
        retain_all!(self.syn_methods, impl_body);
        retain_all!(self.syn_impls, impl_body);
        retain_all!(self.syn_structs, struct_body);
        retain_all!(self.syn_traits, trait_body);
        retain_all!(self.syn_functions, function_body);
        retain_all!(self.syn_tests_mods, tests_mod_body);
        retain_all!(self.syn_enums, enum_body);
        retain_all!(self.syn_unions, union_body);
        retain_all!(self.syn_type_aliases, type_body);
        retain_all!(self.syn_trait_method_sigs, trait_method_signature);
        retain_all!(self.syn_trait_method_defs, trait_method_body);
    }

    /// Kept for callers that still need an owned `FileSynElementsMap` →
    /// `AllSynElements` reconstruction (e.g. serialisation paths).
    pub fn from_file_syn_elements_map(map: FileSynElementsMap) -> Self {
        let mut out = AllSynElements::default();
        for (_file, fse) in map.0 {
            for el in fse.elements {
                match el {
                    AnyFileSynElement::Attribute(x) => out.syn_attributes.push(x),
                    AnyFileSynElement::Method(x) => out.syn_methods.push(x),
                    AnyFileSynElement::Impl(x) => out.syn_impls.push(x),
                    AnyFileSynElement::Struct(x) => out.syn_structs.push(x),
                    AnyFileSynElement::Trait(x) => out.syn_traits.push(x),
                    AnyFileSynElement::Function(x) => out.syn_functions.push(x),
                    AnyFileSynElement::TestsMod(x) => out.syn_tests_mods.push(x),
                    AnyFileSynElement::Enum(x) => out.syn_enums.push(x),
                    AnyFileSynElement::Union(x) => out.syn_unions.push(x),
                    AnyFileSynElement::TypeAlias(x) => out.syn_type_aliases.push(x),
                    AnyFileSynElement::TraitMethodSig(x) => out.syn_trait_method_sigs.push(x),
                    AnyFileSynElement::TraitMethodDef(x) => out.syn_trait_method_defs.push(x),
                    AnyFileSynElement::Sentinel => {}
                }
            }
        }
        out
    }
}
