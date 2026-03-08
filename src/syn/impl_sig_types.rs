//impl_sig_types.rs
use crate::syn::syn_elements::*;
use log::debug;
use std::collections::HashSet;
use syntax_queries::*;
use syntax_queries::byte_range_ordering::{HasByteRange, NodeMatch};

pub type NodeMatches = Vec<NodeMatch>;

pub type NodeMatchesRef<'a> = Option<&'a NodeMatches>;

pub type ForKeywordMatches = NodeMatches;
pub type AsKeywordMatches = NodeMatches;
pub type TypeIdentMatches = NodeMatches;
pub type PrimitiveTypeMatches = NodeMatches;
pub type TypeArgMatches = NodeMatches;
pub type BracketedTypeMatches = NodeMatches;
pub type TraitBoundMatches = NodeMatches;
pub type ConstParamMatches = NodeMatches;
pub type TypeParamMatches = NodeMatches;
pub type WhereClauseMatches = NodeMatches;
pub type IdentifierMatches = NodeMatches;

pub type ForKeywordRef<'a> = Option<&'a ForKeywordMatches>;
pub type TypeIdentRef<'a> = Option<&'a TypeIdentMatches>;
pub type PrimitiveTypeRef<'a> = Option<&'a PrimitiveTypeMatches>;
pub type TypeArgRef<'a> = Option<&'a TypeArgMatches>;
pub type BracketedTypeRef<'a> = Option<&'a BracketedTypeMatches>;
pub type TraitBoundRef<'a> = Option<&'a TraitBoundMatches>;
pub type TypeParamRef<'a> = Option<&'a TypeParamMatches>;
pub type WhereClauseRef<'a> = Option<&'a WhereClauseMatches>;

// pub type IdentifierRef<'a> = Option<&'a IdentifierMatches>;

pub type DSAreasToAvoid<'a> = (
    TypeArgRef<'a>,
    WhereClauseRef<'a>,
    BracketedTypeRef<'a>,
    TypeParamRef<'a>,
);

pub type TypeCandidatesRef<'a> = (TypeIdentRef<'a>, PrimitiveTypeRef<'a>);

pub type DSTypeCandidatesRef<'a> = TypeCandidatesRef<'a>;

pub type TypeSetCtypeExtracted = (Option<TypeVariableMap>, Option<CTypeSet>);

pub struct TypeContext {
    pub type_identifiers: TypeIdentMatches,
    pub primitive_types:  PrimitiveTypeMatches,
}

pub struct TypeContextRef<'a> {
    pub type_identifiers: TypeIdentRef<'a>,
    pub primitive_types:  PrimitiveTypeRef<'a>,
}

// impl TypeContext {
//     pub fn as_ref(&self) -> TypeContextRef<'_> {
//         TypeContextRef {
//             type_identifiers: Some(&self.type_identifiers),
//             primitive_types:  Some(&self.primitive_types),
//         }
//     }
// }

#[allow(dead_code)]
pub struct DsStructureInput<'a> {
    pub for_keyword:     NodeMatchesRef<'a>,
    pub type_parameters: NodeMatchesRef<'a>,
    pub where_clause:    NodeMatchesRef<'a>,
    pub type_arguments:  NodeMatchesRef<'a>,
    pub bracketed_types: NodeMatchesRef<'a>,
}

pub type ExtractionResult = (Option<TypeVariableMap>, Option<CTypeSet>);

macro_rules! impl_collector {
    ($fn_name:ident, $query_name:literal) => {
        pub fn $fn_name(impl_signature: &str) -> Option<Vec<NodeMatch>> {
            match RustParser::new(impl_signature, $query_name) {
                Ok(parser) => parser.find_all(),
                Err(err_str) => {
                    eprintln!(
                        "RustParser::new failed for {} on signature: {}: {}",
                        $query_name, impl_signature, err_str
                    );
                    None
                }
            }
        }
    };
}

impl_collector!(collect_identifiers_from_impl,       "identifier");
impl_collector!(collect_type_identifiers_from_impl,  "type_identifier");
impl_collector!(collect_primitive_types_from_impl,   "primitive_type");
impl_collector!(collect_type_arguments_from_impl,    "type_arguments");
impl_collector!(collect_bracketed_types_from_impl,   "bracketed_type");
impl_collector!(collect_trait_bounds_from_impl,      "trait_bounds");
impl_collector!(collect_const_parameters_from_impl,  "const_parameter");
impl_collector!(collect_for_keyword_from_impl,       "for");
impl_collector!(collect_as_keyword_from_impl,        "as");
impl_collector!(collect_type_parameters_from_impl,   "type_parameters");
impl_collector!(collect_where_clause_from_impl,      "where_clause");

///from_impl_parts
impl TypeIdentifiers {
pub fn from_impl_parts(
    identifiers:     Option<IdentifierMatches>,
    type_context:    TypeContext,
    type_arguments:  Option<TypeArgMatches>,
    bracketed_types: Option<BracketedTypeMatches>,
    trait_bounds:    Option<TraitBoundMatches>,
    const_params:    Option<ConstParamMatches>,
    for_keyword:     Option<ForKeywordMatches>,
    as_keyword:      Option<AsKeywordMatches>,
    type_parameters: Option<TypeParamMatches>,
    where_clause:    Option<WhereClauseMatches>,
) -> (Self, Option<DSName>) {
    let identifiers: NodeMatches = identifiers.unwrap_or_default();
    let TypeContext {
        type_identifiers,
        primitive_types,
    } = type_context;

    let ctx = TypeContextRef {
        type_identifiers: Some(&type_identifiers),
        primitive_types:  Some(&primitive_types),
    };

    // ── raw ds node ───────────────────────────────────────────────────────
    let ds_structure = Self::extract_ds_structure(
        (ctx.type_identifiers, ctx.primitive_types),
        for_keyword.as_ref(),
        (
            type_arguments.as_ref(),
            where_clause.as_ref(),
            bracketed_types.as_ref(),
            type_parameters.as_ref(),
        ),
    );

    // ── collect type variables and concrete types from as keyword (first) ─
    let (ak_type_variables, ak_concrete_types) =
        Self::extract_type_identifiers_from_as_keyword(
            (ctx.type_identifiers, ctx.primitive_types),
            as_keyword.as_ref(),
        );
    let mut type_variables: TypeVariableMap = ak_type_variables.unwrap_or_default();
    let mut concrete_types: CTypeSet        = ak_concrete_types.unwrap_or_default();

    // ── collect type variables and concrete types from trait bounds ────────
    let (tb_type_variables, tb_concrete_types) =
        Self::extract_type_identifiers_from_trait_bounds(
            (ctx.type_identifiers, ctx.primitive_types),
            trait_bounds.as_ref(),
        );
    Self::merge_type_variable_maps(&mut type_variables, tb_type_variables.unwrap_or_default());
    Self::merge_concrete_type_sets(&mut concrete_types, tb_concrete_types.unwrap_or_default());

    // ── collect type variables and concrete types from const params ────────
    let (cp_type_variables, cp_concrete_types) = Self::extract_type_identifiers_from_const(
        (ctx.type_identifiers, ctx.primitive_types),
        const_params.as_ref(),
        Some(&identifiers),
    );
    Self::merge_type_variable_maps(&mut type_variables, cp_type_variables.unwrap_or_default());
    Self::merge_concrete_type_sets(&mut concrete_types, cp_concrete_types.unwrap_or_default());

    // ── inline type-variable references across all TypeSets ───────────────
    Self::type_resolution(&mut type_variables);

    // ── promote speculative concrete types that turned out to be type variables
    let type_var_names: CTypeSet = type_variables.keys().cloned().collect();
    for name in &type_var_names {
        concrete_types.remove(name);
    }

    // ── add remaining type identifiers and primitive types that are not variables
    for ti in type_identifiers
        .iter()
        .filter(|ti| !type_var_names.contains(ti.text.as_str()))
    {
        concrete_types.insert(ti.text.clone());
    }
    for pt in primitive_types
        .iter()
        .filter(|pt| !type_var_names.contains(pt.text.as_str()))
    {
        concrete_types.insert(pt.text.clone());
    }

    // ── resolve ds_structure ──────────────────────────────────────────────
    let ds_structure_text: Option<DSName> =
        ds_structure.as_ref().map(|ds| ds.text.clone()).map(|raw| {
            type_variables
                .get(&raw)
                .and_then(|bounds: &TypeSet| bounds.iter().next())
                .cloned()
                .unwrap_or(raw)
        });

    if let Some(ref ds_text) = ds_structure_text {
        concrete_types.remove(ds_text);
    }

    (
        TypeIdentifiers {
            type_variables: if type_variables.is_empty() { None } else { Some(type_variables) },
            concrete_types: if concrete_types.is_empty() { None } else { Some(concrete_types) },
        },
        ds_structure_text,
    )
}
}

///from_impl_signature
impl TypeIdentifiers {
    pub fn from_impl_signature(impl_signature: &impl ToString) -> (Self, Option<String>) {
        let sig = impl_signature.to_string();

        let type_context = TypeContext {
            type_identifiers: collect_type_identifiers_from_impl(&sig).unwrap_or_default(),
            primitive_types:  collect_primitive_types_from_impl(&sig).unwrap_or_default(),
        };

        Self::from_impl_parts(
            collect_identifiers_from_impl(&sig),
            type_context,
            collect_type_arguments_from_impl(&sig),
            collect_bracketed_types_from_impl(&sig),
            collect_trait_bounds_from_impl(&sig),
            collect_const_parameters_from_impl(&sig),
            collect_for_keyword_from_impl(&sig),
            collect_as_keyword_from_impl(&sig),
            collect_type_parameters_from_impl(&sig),
            collect_where_clause_from_impl(&sig),
        )
    }
}

/// extract type identifiers from trait bounds
impl TypeIdentifiers {
    fn extract_type_identifiers_from_trait_bounds(
        ctx: TypeCandidatesRef<'_>,
        trait_bounds: TraitBoundRef<'_>,
    ) -> TypeSetCtypeExtracted {
        let (type_identifiers, primitive_types) = ctx;

        let type_identifiers_flat: Vec<&NodeMatch> =
            type_identifiers.into_iter().flatten().collect();
        let primitive_types_flat: Vec<&NodeMatch> = primitive_types.into_iter().flatten().collect();
        let mut type_variables: TypeVariableMap = TypeVariableMap::new();
        let mut concrete_types: CTypeSet = CTypeSet::new();

        debug!(
            "[extract_type_identifiers_from_trait_bounds] INPUT:\n  \
         trait_bounds={:#?}\n  \
         type_identifiers_flat={:#?}\n  \
         primitive_types_flat={:#?}",
            trait_bounds, type_identifiers_flat, primitive_types_flat,
        );

        for tb in trait_bounds.iter().copied().flatten() {
            let Some(type_var) = NodeMatch::immediate_before(&type_identifiers_flat, tb) else {
                debug!(
                    "[extract_type_identifiers_from_trait_bounds] SKIP: no type variable found immediately before trait bound {:#?}",
                    tb
                );
                continue;
            };

            debug!(
                "[extract_type_identifiers_from_trait_bounds] MATCH: trait_bound={:#?} -> type_var={:#?}",
                tb, type_var
            );

            let entry = type_variables.entry(type_var.text.clone()).or_default();

            for ti in type_identifiers_flat.iter().filter(|ti| tb.contains(*ti)) {
                debug!(
                    "[extract_type_identifiers_from_trait_bounds] INSERT type_identifier: {:?} into type_var={:?}",
                    ti.text, type_var.text
                );
                entry.insert(ti.text.clone());
                concrete_types.insert(ti.text.clone());
            }

            for pt in primitive_types_flat.iter().filter(|pt| tb.contains(*pt)) {
                debug!(
                    "[extract_type_identifiers_from_trait_bounds] INSERT primitive_type: {:?} into type_var={:?}",
                    pt.text, type_var.text
                );
                entry.insert(pt.text.clone());
                concrete_types.insert(pt.text.clone());
            }
        }

        debug!(
            "[extract_type_identifiers_from_trait_bounds] OUTPUT:\n  \
         type_variables={:#?}\n  \
         concrete_types={:#?}",
            type_variables, concrete_types,
        );

        (Some(type_variables), Some(concrete_types))
    }
}

///extract ds structure
impl TypeIdentifiers {
    fn extract_ds_structure(
        ctx: DSTypeCandidatesRef<'_>,
        for_keyword: ForKeywordRef<'_>,
        areas_to_avoid: DSAreasToAvoid<'_>,
    ) -> Option<NodeMatch> {
        let (type_identifiers, primitive_types) = ctx;
        let (type_arguments, where_clause, bracketed_types, type_parameters) = areas_to_avoid;

        // debug!(
        //     "[extract_ds_structure] INPUT:\n  \
        //      for_keyword={:#?}\n  \
        //      type_parameters={:#?}\n  \
        //      type_identifiers={:#?}\n  \
        //      primitive_types={:#?}\n  \
        //      where_clause={:#?}\n  \
        //      type_arguments={:#?}\n  \
        //      bracketed_types={:#?}",
        //     for_keyword,
        //     type_parameters,
        //     type_identifiers,
        //     primitive_types,
        //     where_clause,
        //     type_arguments,
        //     bracketed_types,
        // );

        // Reference-only collections — no cloning
        let type_identifiers_flat: Vec<&NodeMatch> =
            type_identifiers.into_iter().flatten().collect();
        let primitive_types_flat: Vec<&NodeMatch> = primitive_types.into_iter().flatten().collect();
        let type_params_flat: Vec<&NodeMatch> = type_parameters.into_iter().flatten().collect();
        let base_exclusions: Vec<&NodeMatch> = [where_clause, type_arguments, bracketed_types]
            .iter()
            .copied()
            .flatten()
            .flat_map(|v: &NodeMatches| v.iter())
            .collect();

        // node is &&NodeMatch here; deref once so contains receives &NodeMatch
        let in_base_exclusions =
            |node: &&NodeMatch| -> bool { base_exclusions.iter().any(|c| c.contains(*node)) };

        let valid_for: Vec<&NodeMatch> = for_keyword
            .into_iter()
            .flatten()
            .filter(|fk| {
                !in_base_exclusions(fk) && !type_params_flat.iter().any(|tp| tp.contains(*fk))
            })
            .collect();

        // immediate_after requires &[NodeMatch], so clone only the filtered subsets
        let valid_type_identifiers: Vec<NodeMatch> = type_identifiers_flat
            .iter()
            .filter(|n| !in_base_exclusions(n))
            .copied()
            .cloned()
            .collect();
        let valid_primitive_types: Vec<NodeMatch> = primitive_types_flat
            .iter()
            .filter(|n| !in_base_exclusions(n))
            .copied()
            .cloned()
            .collect();

        // debug!(
        //     "[extract_ds_structure] DERIVED:\n  \
        //      valid_for={:#?}\n  \
        //      valid_type_identifiers={:#?}\n  \
        //      valid_primitive_types={:#?}",
        //     valid_for, valid_type_identifiers, valid_primitive_types,
        // );

        fn pick_earliest<'a>(
            a: Option<&'a NodeMatch>,
            b: Option<&'a NodeMatch>,
        ) -> Option<&'a NodeMatch> {
            match (a, b) {
                (Some(a), Some(b)) => {
                    if a.byte_range().start <= b.byte_range().start {
                        Some(a)
                    } else {
                        Some(b)
                    }
                }
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
        }

        let result = if let Some(fk) = valid_for.first().copied() {
            // debug!(
            //     "[extract_ds_structure] BRANCH: valid `for` keyword found: {:#?}",
            //     fk
            // );
            let next_ti = NodeMatch::immediate_after(&valid_type_identifiers, fk);
            let next_pt = NodeMatch::immediate_after(&valid_primitive_types, fk);
            pick_earliest(next_ti, next_pt).cloned()
        } else if let Some(last_tp) = type_params_flat.last().copied() {
            // debug!(
            //     "[extract_ds_structure] BRANCH: type parameters present, last={:#?}",
            //     last_tp
            // );
            let next_ti = NodeMatch::immediate_after(&valid_type_identifiers, last_tp);
            let next_pt = NodeMatch::immediate_after(&valid_primitive_types, last_tp);
            pick_earliest(next_ti, next_pt).cloned()
        } else {
            // debug!("[extract_ds_structure] BRANCH: fallback to first valid type/primitive");
            let first_ti = valid_type_identifiers
                .iter()
                .min_by_key(|n| n.byte_range().start);
            let first_pt = valid_primitive_types
                .iter()
                .min_by_key(|n| n.byte_range().start);
            pick_earliest(first_ti, first_pt).cloned()
        };

        // debug!("[extract_ds_structure] OUTPUT: {:#?}", result);
        result
    }
}

///extract type identifiers from as keyword
impl TypeIdentifiers {
    fn extract_type_identifiers_from_as_keyword(
        ctx: TypeCandidatesRef<'_>,
        as_keyword: Option<&NodeMatches>,
    ) -> ExtractionResult {
        let type_identifiers_flat: Vec<&NodeMatch> =
            ctx.0.into_iter().flatten().collect();
        let primitive_types_flat: Vec<&NodeMatch> =
            ctx.1.into_iter().flatten().collect();
        let mut type_variables: TypeVariableMap = TypeVariableMap::new();
        let mut concrete_types: CTypeSet = CTypeSet::new();
        for ak in as_keyword.iter().copied().flatten() {
            let Some(type_var) = NodeMatch::immediate_after(&type_identifiers_flat, ak) else {
                continue;
            };
            let bound_type_ti: Option<&&NodeMatch> =
                NodeMatch::immediate_before(&type_identifiers_flat, ak);
            let bound_type_pt: Option<&&NodeMatch> =
                NodeMatch::immediate_before(&primitive_types_flat, ak);
            let bound_type = match (bound_type_ti, bound_type_pt) {
                (Some(ti), Some(pt)) => {
                    if ti.byte_range().start >= pt.byte_range().start {
                        Some(ti)
                    } else {
                        Some(pt)
                    }
                }
                (Some(ti), None) => Some(ti),
                (None, Some(pt)) => Some(pt),
                (None, None) => None,
            };
            let Some(bound) = bound_type else {
                continue;
            };
            let entry = type_variables.entry(type_var.text.clone()).or_default();
            entry.insert(bound.text.clone());
            concrete_types.insert(bound.text.clone());
        }
        (Some(type_variables), Some(concrete_types))
    }
}

///extract type identifiers from const
impl TypeIdentifiers {
    fn extract_type_identifiers_from_const(
        ctx: TypeCandidatesRef<'_>,
        const_params: Option<&NodeMatches>,
        identifiers: Option<&NodeMatches>,
    ) -> ExtractionResult {
        let (type_identifiers, primitive_types) = ctx;
        let identifiers_flat: Vec<NodeMatch> =
            identifiers.into_iter().flatten().cloned().collect();
        let type_identifiers_flat: Vec<NodeMatch> =
            type_identifiers.into_iter().flatten().cloned().collect();
        let primitive_types_flat: Vec<NodeMatch> =
            primitive_types.into_iter().flatten().cloned().collect();
        let mut type_variables: TypeVariableMap = TypeVariableMap::new();
        let mut concrete_types: CTypeSet = CTypeSet::new();
        for cp in const_params.iter().copied().flatten() {
            let first_type_id = NodeMatch::first_contained(&type_identifiers_flat, cp);
            let first_id = NodeMatch::first_contained(&identifiers_flat, cp);
            let type_var = match (first_id, first_type_id) {
                (Some(id), Some(ti)) if id.before(ti) => id,
                (_, Some(ti)) => ti,
                (Some(id), None) if primitive_types_flat.iter().any(|pt| cp.contains(pt)) => id,
                _ => continue,
            };
            let entry = type_variables.entry(type_var.text.clone()).or_default();
            for ti in type_identifiers_flat
                .iter()
                .filter(|ti| cp.contains(*ti) && ti.text != type_var.text)
            {
                entry.insert(ti.text.clone());
                concrete_types.insert(ti.text.clone());
            }
            for pt in primitive_types_flat.iter().filter(|pt| cp.contains(*pt)) {
                entry.insert(pt.text.clone());
                concrete_types.insert(pt.text.clone());
            }
        }
        (Some(type_variables), Some(concrete_types))
    }
}

///type_resolution
impl TypeIdentifiers {
    /// Removes any self-referential entry from every TypeSet, i.e. ensures that
    /// a type variable's own name is never a member of its own bound set.
    ///
    /// Example: `E -> {Into, Error, E}` becomes `E -> {Into, Error}`
    fn remove_self_references(type_variables: &mut TypeVariableMap) {
        for (var, set) in type_variables.iter_mut() {
            set.remove(var.as_str());
        }
    }

    /// Resolves type-variable references inside each variable's TypeSet by
    /// substituting them with the referenced variable's own (concrete) TypeSet.
    ///
    /// Processing order: variables whose sets have the fewest unresolved
    /// type-variable references are expanded first, so a chain like
    ///   S -> {Sink, E, u8}   E -> {Into, Error}
    /// becomes
    ///   S -> {Sink, Into, Error, u8}   E -> {Into, Error}
    fn type_resolution(type_variables: &mut TypeVariableMap) {
        Self::remove_self_references(type_variables);

        let all_var_names: CTypeSet = type_variables.keys().cloned().collect();
        let mut expanded: HashSet<TypeVariable> = HashSet::new();

        loop {
            let mut candidates: Vec<(TypeVariable, usize)> = type_variables
                .iter()
                .filter(|(k, _)| !expanded.contains(*k))
                .map(|(k, v)| {
                    let unresolved_tv_refs = v
                        .iter()
                        .filter(|t| {
                            all_var_names.contains(*t)
                                && !expanded.contains(*t)
                                && *t != k
                        })
                        .count();
                    (k.clone(), unresolved_tv_refs)
                })
                .collect();

            candidates.sort_by_key(|(_, count)| *count);

            let to_expand: Vec<TypeVariable> = candidates
                .into_iter()
                .take_while(|(_, count)| *count == 0)
                .map(|(k, _)| k)
                .collect();

            if to_expand.is_empty() {
                break;
            }

            for var in &to_expand {
                let resolved_set: TypeSet = type_variables[var].clone();
                for (other_var, other_set) in type_variables.iter_mut() {
                    if other_var == var {
                        continue;
                    }
                    if other_set.remove(var.as_str()) {
                        other_set.extend(resolved_set.iter().cloned());
                    }
                }
                expanded.insert(var.clone());
            }
        }

        // A substitution step can re-introduce self-references when a chain
        // eventually resolves back to the variable itself, so clean up once more.
        Self::remove_self_references(type_variables);
    }
}

impl TypeIdentifiers {
    fn merge_type_variable_maps(base: &mut TypeVariableMap, other: TypeVariableMap) {
        for (var, set) in other {
            base.entry(var).or_default().extend(set);
        }
    }

    fn merge_concrete_type_sets(base: &mut CTypeSet, other: CTypeSet) {
        base.extend(other);
    }
}

// Given the for keyword node (if present), find whichever node comes
// immediately after it — whether that is a type identifier or a primitive type.
// fn find_ds_structure<'a>(
//     for_keyword: &[NodeMatch],
//     type_identifiers: &'a [NodeMatch],
//     primitive_types: &'a [NodeMatch],
// ) -> Option<&'a NodeMatch> {
//     // Take the first for keyword, if any
//     let for_kw = for_keyword.first()?;
//     // Find the immediate successor in each vec independently
//     let next_type_ident = NodeMatch::immediate_after(type_identifiers, for_kw);
//     let next_primitive   = NodeMatch::immediate_after(primitive_types,   for_kw);
//     match (next_type_ident, next_primitive) {
//         // Both candidates exist — pick whichever starts earlier
//         (Some(ti), Some(pt)) => {
//             if ti.byte_range().start <= pt.byte_range().start {
//                 Some(ti)
//             } else {
//                 Some(pt)
//             }
//         }
//         // Only one side has a candidate
//         (Some(ti), None) => Some(ti),
//         (None, Some(pt)) => Some(pt),
//         // Nothing follows the for keyword at all
//         (None, None)     => None,
//     }
// }
