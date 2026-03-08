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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_type_visitable_ext() {
    // impl<I: Interner, T: TypeVisitable<I>> TypeVisitableExt<I> for T

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
        make_node(": TypeVisitable<I>", 19, 37),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("T", 18, 19),
        make_node("TypeVisitable", 21, 34),
        make_node("I", 35, 36),
        make_node("TypeVisitableExt", 39, 55),
        make_node("I", 56, 57),
        make_node("T", 63, 64),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // T: TypeVisitable<I>
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["TypeVisitable".into(), "I".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from(["Interner".into(), "TypeVisitable".into(), "I".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_visit_opaque_types() {
    // impl<'tcx, VarFn, OutlivesFn> TypeVisitor<TyCtxt<'tcx>>
    //     for VisitOpaqueTypes<'tcx, VarFn, OutlivesFn>
    // where
    //     VarFn: FnOnce() -> FxHashMap<DefId, ty::Variance>,
    //     OutlivesFn: FnOnce() -> OutlivesEnvironment<'tcx>,

    let trait_bounds = vec![
        make_node(": FnOnce() -> FxHashMap<DefId, ty::Variance>", 121, 165),
        make_node(": FnOnce() -> OutlivesEnvironment<'tcx>", 181, 220),
    ];

    let type_identifiers = vec![
        make_node("VarFn", 11, 16),
        make_node("OutlivesFn", 18, 28),
        make_node("TypeVisitor", 30, 41),
        make_node("TyCtxt", 42, 48),
        make_node("VisitOpaqueTypes", 64, 80),
        make_node("VarFn", 87, 92),
        make_node("OutlivesFn", 94, 104),
        make_node("VarFn", 116, 121),
        make_node("FnOnce", 123, 129),
        make_node("FxHashMap", 135, 144),
        make_node("DefId", 145, 150),
        make_node("Variance", 156, 164),
        make_node("OutlivesFn", 171, 181),
        make_node("FnOnce", 183, 189),
        make_node("OutlivesEnvironment", 195, 214),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // VarFn: FnOnce() -> FxHashMap<DefId, ty::Variance>
    assert_eq!(
        type_variables.get("VarFn").cloned().unwrap_or_default(),
        HashSet::from([
            "FnOnce".into(),
            "FxHashMap".into(),
            "DefId".into(),
            "Variance".into(),
        ])
    );

    // OutlivesFn: FnOnce() -> OutlivesEnvironment<'tcx>
    assert_eq!(
        type_variables.get("OutlivesFn").cloned().unwrap_or_default(),
        HashSet::from(["FnOnce".into(), "OutlivesEnvironment".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "FnOnce".into(),
            "FxHashMap".into(),
            "DefId".into(),
            "Variance".into(),
            "OutlivesEnvironment".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_deep_reject_ctxt_const_params() {
    // impl<I: Interner, const INSTANTIATE_LHS_WITH_INFER: bool, const INSTANTIATE_RHS_WITH_INFER: bool>
    //     DeepRejectCtxt<I, INSTANTIATE_LHS_WITH_INFER, INSTANTIATE_RHS_WITH_INFER>

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("DeepRejectCtxt", 102, 116),
        make_node("I", 117, 118),
        make_node("INSTANTIATE_LHS_WITH_INFER", 120, 146),
        make_node("INSTANTIATE_RHS_WITH_INFER", 148, 174),
    ];

    let primitive_types = vec![
        make_node("bool", 52, 56),
        make_node("bool", 92, 96),
    ];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // Only I: Interner is a trait bound — const params are not trait bounds
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // const params do not appear in type_variables
    assert!(type_variables.get("INSTANTIATE_LHS_WITH_INFER").is_none());
    assert!(type_variables.get("INSTANTIATE_RHS_WITH_INFER").is_none());

    assert_eq!(type_variables.len(), 1);

    // bool is not captured here — it lives inside const params, not trait bounds
    assert_eq!(concrete_types, HashSet::from(["Interner".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_arc_job_fn_send_sync() {
    // impl<BODY> ArcJob<BODY>
    // where
    //     BODY: Fn(JobRefId) + Send + Sync,

    let trait_bounds = vec![
        make_node(": Fn(JobRefId) + Send + Sync", 38, 66),
    ];

    let type_identifiers = vec![
        make_node("BODY", 5, 9),
        make_node("ArcJob", 11, 17),
        make_node("BODY", 18, 22),
        make_node("BODY", 34, 38),
        make_node("Fn", 40, 42),
        make_node("JobRefId", 43, 51),
        make_node("Send", 55, 59),
        make_node("Sync", 62, 66),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // BODY: Fn(JobRefId) + Send + Sync
    assert_eq!(
        type_variables.get("BODY").cloned().unwrap_or_default(),
        HashSet::from(["Fn".into(), "JobRefId".into(), "Send".into(), "Sync".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(
        concrete_types,
        HashSet::from(["Fn".into(), "JobRefId".into(), "Send".into(), "Sync".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_tree_simple_where() {
    // impl<D, R, T> Tree<D, R, T>
    // where
    //     D: Def,
    //     R: Region,
    //     T: Type,

    let trait_bounds = vec![
        make_node(": Def", 39, 44),
        make_node(": Region", 51, 59),
        make_node(": Type", 66, 72),
    ];

    let type_identifiers = vec![
        make_node("D", 5, 6),
        make_node("R", 8, 9),
        make_node("T", 11, 12),
        make_node("Tree", 14, 18),
        make_node("D", 19, 20),
        make_node("R", 22, 23),
        make_node("T", 25, 26),
        make_node("D", 38, 39),
        make_node("Def", 41, 44),
        make_node("R", 50, 51),
        make_node("Region", 53, 59),
        make_node("T", 65, 66),
        make_node("Type", 68, 72),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // D: Def
    assert_eq!(
        type_variables.get("D").cloned().unwrap_or_default(),
        HashSet::from(["Def".into()])
    );

    // R: Region
    assert_eq!(
        type_variables.get("R").cloned().unwrap_or_default(),
        HashSet::from(["Region".into()])
    );

    // T: Type
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["Type".into()])
    );

    assert_eq!(type_variables.len(), 3);

    assert_eq!(
        concrete_types,
        HashSet::from(["Def".into(), "Region".into(), "Type".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_async_write_sink_writer() {
    // impl<S, E, K: SinkWriter, const N: u64> AsyncWrite for K<S>
    // where
    //     for<'a> S: Sink<&'a [u8], Error = E>,
    //     E: Into<io::Error>,

    let trait_bounds = vec![
        make_node(": SinkWriter", 12, 24),
        make_node(": Sink<&'a [u8], Error = E>", 79, 106),
        make_node(": Into<io::Error>", 113, 130),
    ];

    let type_identifiers = vec![
        make_node("S", 5, 6),
        make_node("E", 8, 9),
        make_node("K", 11, 12),
        make_node("SinkWriter", 14, 24),
        make_node("AsyncWrite", 40, 50),
        make_node("K", 55, 56),
        make_node("S", 57, 58),
        make_node("S", 78, 79),
        make_node("Sink", 81, 85),
        make_node("Error", 96, 101),
        make_node("E", 104, 105),
        make_node("E", 112, 113),
        make_node("Into", 115, 119),
        make_node("Error", 124, 129),
    ];

    let primitive_types = vec![
        make_node("u64", 35, 38),
        make_node("u8", 91, 93),
    ];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // K: SinkWriter (inline bound, not in where clause)
    assert_eq!(
        type_variables.get("K").cloned().unwrap_or_default(),
        HashSet::from(["SinkWriter".into()])
    );

    // for<'a> S: Sink<&'a [u8], Error = E>
    // — associated type `Error = E` contributes both "Error" and "E"
    // — primitive `u8` inside the slice ref is also captured
    assert_eq!(
        type_variables.get("S").cloned().unwrap_or_default(),
        HashSet::from(["Sink".into(), "Error".into(), "E".into(), "u8".into()])
    );

    // E: Into<io::Error>  — path-qualified io::Error contributes only "Error"
    assert_eq!(
        type_variables.get("E").cloned().unwrap_or_default(),
        HashSet::from(["Into".into(), "Error".into()])
    );

    assert_eq!(type_variables.len(), 3);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "SinkWriter".into(),
            "Sink".into(),
            "Error".into(),
            "E".into(),
            "u8".into(),
            "Into".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_canonical_fmt_display() {
    // impl<I: Interner, V: fmt::Display> fmt::Display for Canonical<I, V>

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
        make_node(": fmt::Display", 19, 33),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("V", 18, 19),
        make_node("Display", 26, 33),
        make_node("Display", 40, 47),
        make_node("Canonical", 52, 61),
        make_node("I", 62, 63),
        make_node("V", 65, 66),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // V: fmt::Display — path-qualified, only final segment "Display" is captured
    assert_eq!(
        type_variables.get("V").cloned().unwrap_or_default(),
        HashSet::from(["Display".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from(["Interner".into(), "Display".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_canonical_var_values_into_iterator() {
    // impl<'a, I: Interner> IntoIterator for &'a CanonicalVarValues<I>

    let trait_bounds = vec![
        make_node(": Interner", 10, 20),
    ];

    let type_identifiers = vec![
        make_node("I", 9, 10),
        make_node("Interner", 12, 20),
        make_node("IntoIterator", 22, 34),
        make_node("CanonicalVarValues", 43, 61),
        make_node("I", 62, 63),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(concrete_types, HashSet::from(["Interner".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_upcast_from_binder_trait_ref() {
    // impl<I: Interner> UpcastFrom<I, ty::Binder<I, TraitRef<I>>> for ty::Binder<I, TraitPredicate<I>>

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("UpcastFrom", 18, 28),
        make_node("I", 29, 30),
        make_node("Binder", 36, 42),
        make_node("I", 43, 44),
        make_node("TraitRef", 46, 54),
        make_node("I", 55, 56),
        make_node("Binder", 68, 74),
        make_node("I", 75, 76),
        make_node("TraitPredicate", 78, 92),
        make_node("I", 93, 94),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // Only I: Interner — single inline bound, no where clause at all
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(concrete_types, HashSet::from(["Interner".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_solver_relating_infer_ctxt_like() {
    // impl<Infcx, I> TypeRelation<I> for SolverRelating<'_, Infcx, I>
    // where
    //     Infcx: InferCtxtLike<Interner = I>,
    //     I: Interner,

    let trait_bounds = vec![
        make_node(": InferCtxtLike<Interner = I>", 79, 108),
        make_node(": Interner", 115, 125),
    ];

    let type_identifiers = vec![
        make_node("Infcx", 5, 10),
        make_node("I", 12, 13),
        make_node("TypeRelation", 15, 27),
        make_node("I", 28, 29),
        make_node("SolverRelating", 35, 49),
        make_node("Infcx", 54, 59),
        make_node("I", 61, 62),
        make_node("Infcx", 74, 79),
        make_node("InferCtxtLike", 81, 94),
        make_node("Interner", 95, 103),
        make_node("I", 106, 107),
        make_node("I", 114, 115),
        make_node("Interner", 117, 125),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // Infcx: InferCtxtLike<Interner = I>
    // — associated type "Interner" and its assigned value "I" are both captured
    assert_eq!(
        type_variables.get("Infcx").cloned().unwrap_or_default(),
        HashSet::from(["InferCtxtLike".into(), "Interner".into(), "I".into()])
    );

    // I: Interner
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from(["InferCtxtLike".into(), "Interner".into(), "I".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_search_graph_delegate_cx() {
    // impl<D: Delegate<Cx = X>, X: Cx> SearchGraph<D>

    let trait_bounds = vec![
        make_node(": Delegate<Cx = X>", 6, 24),
        make_node(": Cx", 27, 31),
    ];

    let type_identifiers = vec![
        make_node("D", 5, 6),
        make_node("Delegate", 8, 16),
        make_node("Cx", 17, 19),
        make_node("X", 22, 23),
        make_node("X", 26, 27),
        make_node("Cx", 29, 31),
        make_node("SearchGraph", 33, 44),
        make_node("D", 45, 46),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // D: Delegate<Cx = X> — associated type "Cx" and its assigned value "X" both captured
    assert_eq!(
        type_variables.get("D").cloned().unwrap_or_default(),
        HashSet::from(["Delegate".into(), "Cx".into(), "X".into()])
    );

    // X: Cx
    assert_eq!(
        type_variables.get("X").cloned().unwrap_or_default(),
        HashSet::from(["Cx".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from(["Delegate".into(), "Cx".into(), "X".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_binder_ir_print_self_referential() {
    // impl<I: Interner, T> fmt::Display for Binder<I, T>
    // where
    //     I: IrPrint<Binder<I, T>>,

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
        make_node(": IrPrint<Binder<I, T>>", 62, 85),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("T", 18, 19),
        make_node("Display", 26, 33),
        make_node("Binder", 38, 44),
        make_node("I", 45, 46),
        make_node("T", 48, 49),
        make_node("I", 61, 62),
        make_node("IrPrint", 64, 71),
        make_node("Binder", 72, 78),
        make_node("I", 79, 80),
        make_node("T", 82, 83),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner (inline) + I: IrPrint<Binder<I, T>> (where clause)
    // — both bounds merge into the same "I" entry via or_default()
    // — the where clause bound contains I itself as a nested type arg, so "I" appears
    //   in its own bound set
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from([
            "Interner".into(),
            "IrPrint".into(),
            "Binder".into(),
            "I".into(),
            "T".into(),
        ])
    );

    // T is an unbounded type parameter — no entry in type_variables
    assert!(type_variables.get("T").is_none());

    assert_eq!(type_variables.len(), 1);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Interner".into(),
            "IrPrint".into(),
            "Binder".into(),
            "I".into(),
            "T".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_binder_lift_bound_var_kinds() {
    // impl<I: Interner, U: Interner, T> Lift<U> for Binder<I, T>
    // where
    //     T: Lift<U>,
    //     I::BoundVarKinds: Lift<U, Lifted = U::BoundVarKinds>,

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
        make_node(": Interner", 19, 29),
        make_node(": Lift<U>", 70, 79),
        make_node(": Lift<U, Lifted = U::BoundVarKinds>", 101, 137),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("U", 18, 19),
        make_node("Interner", 21, 29),
        make_node("T", 31, 32),
        make_node("Lift", 34, 38),
        make_node("U", 39, 40),
        make_node("Binder", 46, 52),
        make_node("I", 53, 54),
        make_node("T", 56, 57),
        make_node("T", 69, 70),
        make_node("Lift", 72, 76),
        make_node("U", 77, 78),
        make_node("BoundVarKinds", 88, 101),
        make_node("Lift", 103, 107),
        make_node("U", 108, 109),
        make_node("Lifted", 111, 117),
        make_node("BoundVarKinds", 123, 136),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner (inline)
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // U: Interner (inline)
    assert_eq!(
        type_variables.get("U").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // T: Lift<U> — U captured as type arg
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["Lift".into(), "U".into()])
    );

    // I::BoundVarKinds: Lift<U, Lifted = U::BoundVarKinds>
    // — path subject resolves to type_var "BoundVarKinds" (final segment only)
    // — U::BoundVarKinds on rhs contributes only "BoundVarKinds", making it self-referential
    assert_eq!(
        type_variables.get("BoundVarKinds").cloned().unwrap_or_default(),
        HashSet::from([
            "Lift".into(),
            "U".into(),
            "Lifted".into(),
            "BoundVarKinds".into(),
        ])
    );

    assert_eq!(type_variables.len(), 4);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Interner".into(),
            "Lift".into(),
            "U".into(),
            "Lifted".into(),
            "BoundVarKinds".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_iter_instantiated_slice_like() {
    // impl<I: Interner, Iter: IntoIterator, A> Iterator for IterInstantiated<I, Iter, A>
    // where
    //     Iter::Item: TypeFoldable<I>,
    //     A: SliceLike<Item = I::GenericArg>,

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
        make_node(": IntoIterator", 22, 36),
        make_node(": TypeFoldable<I>", 103, 120),
        make_node(": SliceLike<Item = I::GenericArg>", 127, 160),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("Iter", 18, 22),
        make_node("IntoIterator", 24, 36),
        make_node("A", 38, 39),
        make_node("Iterator", 41, 49),
        make_node("IterInstantiated", 54, 70),
        make_node("I", 71, 72),
        make_node("Iter", 74, 78),
        make_node("A", 80, 81),
        make_node("Item", 99, 103),
        make_node("TypeFoldable", 105, 117),
        make_node("I", 118, 119),
        make_node("A", 126, 127),
        make_node("SliceLike", 129, 138),
        make_node("Item", 139, 143),
        make_node("GenericArg", 149, 159),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner (inline)
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // Iter: IntoIterator (inline)
    assert_eq!(
        type_variables.get("Iter").cloned().unwrap_or_default(),
        HashSet::from(["IntoIterator".into()])
    );

    // Iter::Item: TypeFoldable<I>
    // — path subject resolves to type_var "Item" (final segment of Iter::Item)
    // — I captured as type arg inside TypeFoldable<I>
    assert_eq!(
        type_variables.get("Item").cloned().unwrap_or_default(),
        HashSet::from(["TypeFoldable".into(), "I".into()])
    );

    // A: SliceLike<Item = I::GenericArg>
    // — associated type "Item" and rhs path segment "GenericArg" both captured
    // — I is dropped (only final segment of I::GenericArg is captured)
    assert_eq!(
        type_variables.get("A").cloned().unwrap_or_default(),
        HashSet::from(["SliceLike".into(), "Item".into(), "GenericArg".into()])
    );

    assert_eq!(type_variables.len(), 4);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Interner".into(),
            "IntoIterator".into(),
            "TypeFoldable".into(),
            "I".into(),
            "SliceLike".into(),
            "Item".into(),
            "GenericArg".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_iter_instantiated_slice_like() {
    // impl<I: Interner, Iter: IntoIterator, A> Iterator for IterInstantiated<I, Iter, A>
    // where
    //     Iter::Item: TypeFoldable<I>,
    //     A: SliceLike<Item = I::GenericArg>,

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
        make_node(": IntoIterator", 22, 36),
        make_node(": TypeFoldable<I>", 103, 120),
        make_node(": SliceLike<Item = I::GenericArg>", 127, 160),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("Iter", 18, 22),
        make_node("IntoIterator", 24, 36),
        make_node("A", 38, 39),
        make_node("Iterator", 41, 49),
        make_node("IterInstantiated", 54, 70),
        make_node("I", 71, 72),
        make_node("Iter", 74, 78),
        make_node("A", 80, 81),
        make_node("Item", 99, 103),
        make_node("TypeFoldable", 105, 117),
        make_node("I", 118, 119),
        make_node("A", 126, 127),
        make_node("SliceLike", 129, 138),
        make_node("Item", 139, 143),
        make_node("GenericArg", 149, 159),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner (inline)
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // Iter: IntoIterator (inline)
    assert_eq!(
        type_variables.get("Iter").cloned().unwrap_or_default(),
        HashSet::from(["IntoIterator".into()])
    );

    // Iter::Item: TypeFoldable<I>
    // — path subject resolves to type_var "Item" (final segment of Iter::Item)
    // — I captured as type arg inside TypeFoldable<I>
    assert_eq!(
        type_variables.get("Item").cloned().unwrap_or_default(),
        HashSet::from(["TypeFoldable".into(), "I".into()])
    );

    // A: SliceLike<Item = I::GenericArg>
    // — associated type "Item" and rhs path segment "GenericArg" both captured
    // — I is dropped (only final segment of I::GenericArg is captured)
    assert_eq!(
        type_variables.get("A").cloned().unwrap_or_default(),
        HashSet::from(["SliceLike".into(), "Item".into(), "GenericArg".into()])
    );

    assert_eq!(type_variables.len(), 4);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Interner".into(),
            "IntoIterator".into(),
            "TypeFoldable".into(),
            "I".into(),
            "SliceLike".into(),
            "Item".into(),
            "GenericArg".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_region_kind_hash_stable() {
    // impl<CTX, I: Interner> HashStable<CTX> for RegionKind<I>
    // where
    //     I::EarlyParamRegion: HashStable<CTX>,
    //     I::LateParamRegion: HashStable<CTX>,
    //     I::DefId: HashStable<CTX>,
    //     I::Symbol: HashStable<CTX>,

    let trait_bounds = vec![
        make_node(": Interner", 11, 21),
        make_node(": HashStable<CTX>", 86, 103),
        make_node(": HashStable<CTX>", 127, 144),
        make_node(": HashStable<CTX>", 158, 175),
        make_node(": HashStable<CTX>", 190, 207),
    ];

    let type_identifiers = vec![
        make_node("CTX", 5, 8),
        make_node("I", 10, 11),
        make_node("Interner", 13, 21),
        make_node("HashStable", 23, 33),
        make_node("CTX", 34, 37),
        make_node("RegionKind", 43, 53),
        make_node("I", 54, 55),
        make_node("EarlyParamRegion", 70, 86),
        make_node("HashStable", 88, 98),
        make_node("CTX", 99, 102),
        make_node("LateParamRegion", 112, 127),
        make_node("HashStable", 129, 139),
        make_node("CTX", 140, 143),
        make_node("DefId", 153, 158),
        make_node("HashStable", 160, 170),
        make_node("CTX", 171, 174),
        make_node("Symbol", 184, 190),
        make_node("HashStable", 192, 202),
        make_node("CTX", 203, 206),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner (inline) — CTX has no bound so no entry for it
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // All four I::X path projections resolve to their final segment as type_var key,
    // each receiving the identical bound set {HashStable, CTX}
    for key in &["EarlyParamRegion", "LateParamRegion", "DefId", "Symbol"] {
        assert_eq!(
            type_variables.get(*key).cloned().unwrap_or_default(),
            HashSet::from(["HashStable".into(), "CTX".into()]),
            "unexpected bound set for type_var {key}"
        );
    }

    // CTX itself has no bound entry despite appearing as a type arg in every bound
    assert!(type_variables.get("CTX").is_none());

    assert_eq!(type_variables.len(), 5);

    assert_eq!(
        concrete_types,
        HashSet::from(["Interner".into(), "HashStable".into(), "CTX".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_option_arena_allocatable() {
    // impl<'tcx, T: ArenaAllocatable<'tcx>> ProcessQueryValue<'tcx, &'tcx T> for Option<T>

    let trait_bounds = vec![
        make_node(": ArenaAllocatable<'tcx>", 12, 36),
    ];

    let type_identifiers = vec![
        make_node("T", 11, 12),
        make_node("ArenaAllocatable", 14, 30),
        make_node("ProcessQueryValue", 38, 55),
        make_node("T", 68, 69),
        make_node("Option", 75, 81),
        make_node("T", 82, 83),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // T: ArenaAllocatable<'tcx> — lifetime arg is invisible, only ArenaAllocatable captured
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["ArenaAllocatable".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(concrete_types, HashSet::from(["ArenaAllocatable".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_option_decode_iterator() {
    // impl<'tcx, D: Decoder, T: Copy + Decodable<D>> ProcessQueryValue<'tcx, Option<&'tcx [T]>>
    //     for Option<DecodeIterator<T, D>>

    let trait_bounds = vec![
        make_node(": Decoder", 12, 21),
        make_node(": Copy + Decodable<D>", 24, 45),
    ];

    let type_identifiers = vec![
        make_node("D", 11, 12),
        make_node("Decoder", 14, 21),
        make_node("T", 23, 24),
        make_node("Copy", 26, 30),
        make_node("Decodable", 33, 42),
        make_node("D", 43, 44),
        make_node("ProcessQueryValue", 47, 64),
        make_node("Option", 71, 77),
        make_node("T", 85, 86),
        make_node("Option", 98, 104),
        make_node("DecodeIterator", 105, 119),
        make_node("T", 120, 121),
        make_node("D", 123, 124),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // D: Decoder
    assert_eq!(
        type_variables.get("D").cloned().unwrap_or_default(),
        HashSet::from(["Decoder".into()])
    );

    // T: Copy + Decodable<D>
    // — compound + bound: both Copy and Decodable captured
    // — D captured as type arg inside Decodable<D>, making D appear in both
    //   its own entry key and in T's bound set
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["Copy".into(), "Decodable".into(), "D".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Decoder".into(),
            "Copy".into(),
            "Decodable".into(),
            "D".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_table_builder_fixed_size_encoding() {
    // impl<I: Idx, const N: usize, T: FixedSizeEncoding<ByteArray = [u8; N]>> TableBuilder<I, T>

    let trait_bounds = vec![
        make_node(": Idx", 6, 11),
        make_node(": FixedSizeEncoding<ByteArray = [u8; N]>", 30, 70),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Idx", 8, 11),
        make_node("T", 29, 30),
        make_node("FixedSizeEncoding", 32, 49),
        make_node("ByteArray", 50, 59),
        make_node("TableBuilder", 72, 84),
        make_node("I", 85, 86),
        make_node("T", 88, 89),
    ];

    let primitive_types = vec![
        make_node("usize", 22, 27),
        make_node("u8", 63, 65),
    ];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Idx (inline)
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Idx".into()])
    );

    // T: FixedSizeEncoding<ByteArray = [u8; N]>
    // — associated type "ByteArray" captured as identifier
    // — u8 inside the array expression [u8; N] captured as primitive type
    // — const N is not a type identifier so it produces no NodeMatch and is invisible
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["FixedSizeEncoding".into(), "ByteArray".into(), "u8".into()])
    );

    // const N: usize — usize is a primitive in the const param list, not inside a bound,
    // so it falls outside any trait bound node and is never captured here
    assert!(type_variables.get("N").is_none());

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Idx".into(),
            "FixedSizeEncoding".into(),
            "ByteArray".into(),
            "u8".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_sink_writer_async_write_no_inline_bounds() {
    // impl<S, E> AsyncWrite for SinkWriter<S>
    // where
    //     for<'a> S: Sink<&'a [u8], Error = E>,
    //     E: Into<io::Error>,

    let trait_bounds = vec![
        make_node(": Sink<&'a [u8], Error = E>", 59, 86),
        make_node(": Into<io::Error>", 93, 110),
    ];

    let type_identifiers = vec![
        make_node("S", 5, 6),
        make_node("E", 8, 9),
        make_node("AsyncWrite", 11, 21),
        make_node("SinkWriter", 26, 36),
        make_node("S", 37, 38),
        make_node("S", 58, 59),
        make_node("Sink", 61, 65),
        make_node("Error", 76, 81),
        make_node("E", 84, 85),
        make_node("E", 92, 93),
        make_node("Into", 95, 99),
        make_node("Error", 104, 109),
    ];

    let primitive_types = vec![
        make_node("u8", 71, 73),
    ];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // for<'a> S: Sink<&'a [u8], Error = E>
    // — HRTB lifetime invisible, u8 inside slice captured as primitive,
    //   associated type "Error" and its value "E" both captured
    assert_eq!(
        type_variables.get("S").cloned().unwrap_or_default(),
        HashSet::from(["Sink".into(), "Error".into(), "E".into(), "u8".into()])
    );

    // E: Into<io::Error> — path-qualified, only "Error" segment captured
    assert_eq!(
        type_variables.get("E").cloned().unwrap_or_default(),
        HashSet::from(["Into".into(), "Error".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Sink".into(),
            "Error".into(),
            "E".into(),
            "u8".into(),
            "Into".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_backtrace_formatter_lookup_span() {
    // impl<S, N> FormatEvent<S, N> for BacktraceFormatter
    // where
    //     S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    //     N: for<'a> FormatFields<'a> + 'static,

    let trait_bounds = vec![
        make_node(": Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>", 63, 130),
        make_node(": for<'a> FormatFields<'a> + 'static", 137, 173),
    ];

    let type_identifiers = vec![
        make_node("S", 5, 6),
        make_node("N", 8, 9),
        make_node("FormatEvent", 11, 22),
        make_node("S", 23, 24),
        make_node("N", 26, 27),
        make_node("BacktraceFormatter", 33, 51),
        make_node("S", 62, 63),
        make_node("Subscriber", 65, 75),
        make_node("LookupSpan", 116, 126),
        make_node("N", 136, 137),
        make_node("FormatFields", 147, 159),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>
    // — compound + bound with an inline HRTB on the second component
    // — path-qualified LookupSpan: only final segment captured, 'a arg invisible
    assert_eq!(
        type_variables.get("S").cloned().unwrap_or_default(),
        HashSet::from(["Subscriber".into(), "LookupSpan".into()])
    );

    // N: for<'a> FormatFields<'a> + 'static
    // — HRTB on the primary bound itself, lifetime arg 'a invisible
    // — 'static is a lifetime bound, not a type identifier, so it produces no NodeMatch
    assert_eq!(
        type_variables.get("N").cloned().unwrap_or_default(),
        HashSet::from(["FormatFields".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from(["Subscriber".into(), "LookupSpan".into(), "FormatFields".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_fact_row_tuple_impl() {
    // impl<A, B> FactRow for (A, B)
    // where
    //     A: FactCell,
    //     B: FactCell,

    let trait_bounds = vec![
        make_node(": FactCell", 41, 51),
        make_node(": FactCell", 58, 68),
    ];

    let type_identifiers = vec![
        make_node("A", 5, 6),
        make_node("B", 8, 9),
        make_node("FactRow", 11, 18),
        make_node("A", 24, 25),
        make_node("B", 27, 28),
        make_node("A", 40, 41),
        make_node("FactCell", 43, 51),
        make_node("B", 57, 58),
        make_node("FactCell", 60, 68),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // A: FactCell
    assert_eq!(
        type_variables.get("A").cloned().unwrap_or_default(),
        HashSet::from(["FactCell".into()])
    );

    // B: FactCell
    assert_eq!(
        type_variables.get("B").cloned().unwrap_or_default(),
        HashSet::from(["FactCell".into()])
    );

    assert_eq!(type_variables.len(), 2);

    // Both type vars share the same bound — concrete_types deduplicates to a single entry
    assert_eq!(concrete_types, HashSet::from(["FactCell".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_unord_items_iterator_item() {
    // impl<T, I: Iterator<Item = T>> UnordItems<T, I>

    let trait_bounds = vec![
        make_node(": Iterator<Item = T>", 9, 29),
    ];

    let type_identifiers = vec![
        make_node("T", 5, 6),
        make_node("I", 8, 9),
        make_node("Iterator", 11, 19),
        make_node("Item", 20, 24),
        make_node("T", 27, 28),
        make_node("UnordItems", 31, 41),
        make_node("T", 42, 43),
        make_node("I", 45, 46),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Iterator<Item = T>
    // — associated type "Item" captured, its value "T" also captured
    // — T is an unbounded type param that appears only as the rhs of Item = T,
    //   so it enters I's bound set and concrete_types solely via this association
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Iterator".into(), "Item".into(), "T".into()])
    );

    // T itself has no bound — no entry in type_variables
    assert!(type_variables.get("T").is_none());

    assert_eq!(type_variables.len(), 1);

    assert_eq!(
        concrete_types,
        HashSet::from(["Iterator".into(), "Item".into(), "T".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_fingerprint_hasher_for_h() {
    // impl<H: Hasher> FingerprintHasher for H

    let trait_bounds = vec![
        make_node(": Hasher", 6, 14),
    ];

    let type_identifiers = vec![
        make_node("H", 5, 6),
        make_node("Hasher", 8, 14),
        make_node("FingerprintHasher", 16, 33),
        make_node("H", 38, 39),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // H: Hasher — single inline bound, no where clause
    assert_eq!(
        type_variables.get("H").cloned().unwrap_or_default(),
        HashSet::from(["Hasher".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(concrete_types, HashSet::from(["Hasher".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_delegate_for_mut_ref() {
    // impl<'tcx, D: Delegate<'tcx>> Delegate<'tcx> for &mut D

    let trait_bounds = vec![
        make_node(": Delegate<'tcx>", 12, 28),
    ];

    let type_identifiers = vec![
        make_node("D", 11, 12),
        make_node("Delegate", 14, 22),
        make_node("Delegate", 30, 38),
        make_node("D", 54, 55),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // D: Delegate<'tcx> — lifetime-only type arg, so only "Delegate" captured
    assert_eq!(
        type_variables.get("D").cloned().unwrap_or_default(),
        HashSet::from(["Delegate".into()])
    );

    assert_eq!(type_variables.len(), 1);

    // "Delegate" appears twice in type_identifiers_flat (once as the bound at 14–22,
    // once as the trait being implemented at 30–38) but concrete_types deduplicates to one
    assert_eq!(concrete_types, HashSet::from(["Delegate".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_union_iter_tuple_item() {
    // impl<S: Copy, X: Iterator<Item = (Byte, S)>, Y: Iterator<Item = (Byte, S)>> Iterator
    //     for UnionIter<X, Y>

    let trait_bounds = vec![
        make_node(": Copy", 6, 12),
        make_node(": Iterator<Item = (Byte, S)>", 15, 43),
        make_node(": Iterator<Item = (Byte, S)>", 46, 74),
    ];

    let type_identifiers = vec![
        make_node("S", 5, 6),
        make_node("Copy", 8, 12),
        make_node("X", 14, 15),
        make_node("Iterator", 17, 25),
        make_node("Item", 26, 30),
        make_node("Byte", 34, 38),
        make_node("S", 40, 41),
        make_node("Y", 45, 46),
        make_node("Iterator", 48, 56),
        make_node("Item", 57, 61),
        make_node("Byte", 65, 69),
        make_node("S", 71, 72),
        make_node("Iterator", 76, 84),
        make_node("UnionIter", 93, 102),
        make_node("X", 103, 104),
        make_node("Y", 106, 107),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // S: Copy
    assert_eq!(
        type_variables.get("S").cloned().unwrap_or_default(),
        HashSet::from(["Copy".into()])
    );

    // X: Iterator<Item = (Byte, S)>
    // — associated type value is a tuple (Byte, S): both Byte and S captured
    // — S is another type parameter, entering X's bound set via the tuple value
    assert_eq!(
        type_variables.get("X").cloned().unwrap_or_default(),
        HashSet::from(["Iterator".into(), "Item".into(), "Byte".into(), "S".into()])
    );

    // Y: Iterator<Item = (Byte, S)> — identical bound to X, same result
    assert_eq!(
        type_variables.get("Y").cloned().unwrap_or_default(),
        HashSet::from(["Iterator".into(), "Item".into(), "Byte".into(), "S".into()])
    );

    assert_eq!(type_variables.len(), 3);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Copy".into(),
            "Iterator".into(),
            "Item".into(),
            "Byte".into(),
            "S".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_rustc_peek_at_analysis() {
    // impl<'tcx, A, D> RustcPeekAt<'tcx> for A
    // where
    //     A: Analysis<'tcx, Domain = D> + HasMoveData<'tcx>,
    //     D: JoinSemiLattice + Clone + BitSetExt<MovePathIndex>,

    let trait_bounds = vec![
        make_node(": Analysis<'tcx, Domain = D> + HasMoveData<'tcx>", 52, 100),
        make_node(": JoinSemiLattice + Clone + BitSetExt<MovePathIndex>", 107, 159),
    ];

    let type_identifiers = vec![
        make_node("A", 11, 12),
        make_node("D", 14, 15),
        make_node("RustcPeekAt", 17, 28),
        make_node("A", 39, 40),
        make_node("A", 51, 52),
        make_node("Analysis", 54, 62),
        make_node("Domain", 69, 75),
        make_node("D", 78, 79),
        make_node("HasMoveData", 83, 94),
        make_node("D", 106, 107),
        make_node("JoinSemiLattice", 109, 124),
        make_node("Clone", 127, 132),
        make_node("BitSetExt", 135, 144),
        make_node("MovePathIndex", 145, 158),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // A: Analysis<'tcx, Domain = D> + HasMoveData<'tcx>
    // — compound + bound with lifetime args (invisible), associated type "Domain" and
    //   its value "D" captured, second compound component "HasMoveData" captured
    assert_eq!(
        type_variables.get("A").cloned().unwrap_or_default(),
        HashSet::from([
            "Analysis".into(),
            "Domain".into(),
            "D".into(),
            "HasMoveData".into(),
        ])
    );

    // D: JoinSemiLattice + Clone + BitSetExt<MovePathIndex>
    // — three-component compound bound, MovePathIndex captured as type arg to BitSetExt
    assert_eq!(
        type_variables.get("D").cloned().unwrap_or_default(),
        HashSet::from([
            "JoinSemiLattice".into(),
            "Clone".into(),
            "BitSetExt".into(),
            "MovePathIndex".into(),
        ])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Analysis".into(),
            "Domain".into(),
            "D".into(),
            "HasMoveData".into(),
            "JoinSemiLattice".into(),
            "Clone".into(),
            "BitSetExt".into(),
            "MovePathIndex".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_formatter_labeller_domain_debug() {
    // impl<'tcx, A> dot::Labeller<'_> for Formatter<'_, 'tcx, A>
    // where
    //     A: Analysis<'tcx>,
    //     A::Domain: DebugWithContext<A>,

    let trait_bounds = vec![
        make_node(": Analysis<'tcx>", 70, 86),
        make_node(": DebugWithContext<A>", 101, 122),
    ];

    let type_identifiers = vec![
        make_node("A", 11, 12),
        make_node("Labeller", 19, 27),
        make_node("Formatter", 36, 45),
        make_node("A", 56, 57),
        make_node("A", 69, 70),
        make_node("Analysis", 72, 80),
        make_node("Domain", 95, 101),
        make_node("DebugWithContext", 103, 119),
        make_node("A", 120, 121),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // A: Analysis<'tcx> — lifetime arg invisible, only Analysis captured
    assert_eq!(
        type_variables.get("A").cloned().unwrap_or_default(),
        HashSet::from(["Analysis".into()])
    );

    // A::Domain: DebugWithContext<A>
    // — path subject resolves to type_var "Domain" (final segment of A::Domain)
    // — "A" captured as type arg inside DebugWithContext<A>, making the bounded
    //   type param "A" appear in "Domain"'s entry and in concrete_types
    assert_eq!(
        type_variables.get("Domain").cloned().unwrap_or_default(),
        HashSet::from(["DebugWithContext".into(), "A".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from(["Analysis".into(), "DebugWithContext".into(), "A".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_move_path_index_has_move_data() {
    // impl<'tcx, C> DebugWithContext<C> for crate::move_paths::MovePathIndex
    // where
    //     C: crate::move_paths::HasMoveData<'tcx>,

    let trait_bounds = vec![
        make_node(": crate::move_paths::HasMoveData<'tcx>", 82, 120),
    ];

    let type_identifiers = vec![
        make_node("C", 11, 12),
        make_node("DebugWithContext", 14, 30),
        make_node("C", 31, 32),
        make_node("MovePathIndex", 57, 70),
        make_node("C", 81, 82),
        make_node("HasMoveData", 103, 114),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // C: crate::move_paths::HasMoveData<'tcx>
    // — deeply path-qualified bound: only final segment "HasMoveData" captured
    // — lifetime arg 'tcx invisible
    assert_eq!(
        type_variables.get("C").cloned().unwrap_or_default(),
        HashSet::from(["HasMoveData".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(concrete_types, HashSet::from(["HasMoveData".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_mixed_bit_set_debug_with_context() {
    // impl<T, C> DebugWithContext<C> for MixedBitSet<T>
    // where
    //     T: Idx + DebugWithContext<C>,

    let trait_bounds = vec![
        make_node(": Idx + DebugWithContext<C>", 61, 88),
    ];

    let type_identifiers = vec![
        make_node("T", 5, 6),
        make_node("C", 8, 9),
        make_node("DebugWithContext", 11, 27),
        make_node("C", 28, 29),
        make_node("MixedBitSet", 35, 46),
        make_node("T", 47, 48),
        make_node("T", 60, 61),
        make_node("Idx", 63, 66),
        make_node("DebugWithContext", 69, 85),
        make_node("C", 86, 87),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // T: Idx + DebugWithContext<C>
    // — compound bound: Idx and DebugWithContext both captured
    // — C is an unbounded type parameter captured as type arg inside DebugWithContext<C>
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["Idx".into(), "DebugWithContext".into(), "C".into()])
    );

    // C has no bound of its own — no entry in type_variables
    assert!(type_variables.get("C").is_none());

    assert_eq!(type_variables.len(), 1);

    assert_eq!(
        concrete_types,
        HashSet::from(["Idx".into(), "DebugWithContext".into(), "C".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_generic_cx_borrow_scx() {
    // impl<'ll, CX: Borrow<SCx<'ll>>> GenericCx<'ll, CX>

    let trait_bounds = vec![
        make_node(": Borrow<SCx<'ll>>", 12, 30),
    ];

    let type_identifiers = vec![
        make_node("CX", 10, 12),
        make_node("Borrow", 14, 20),
        make_node("SCx", 21, 24),
        make_node("GenericCx", 32, 41),
        make_node("CX", 47, 49),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // CX: Borrow<SCx<'ll>>
    // — nested generic type arg: SCx is captured as the type arg to Borrow,
    //   its own lifetime arg 'll is invisible
    assert_eq!(
        type_variables.get("CX").cloned().unwrap_or_default(),
        HashSet::from(["Borrow".into(), "SCx".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(
        concrete_types,
        HashSet::from(["Borrow".into(), "SCx".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_encodable_slice_impl() {
    // impl<S: Encoder, T: Encodable<S>> Encodable<S> for [T]

    let trait_bounds = vec![
        make_node(": Encoder", 6, 15),
        make_node(": Encodable<S>", 18, 32),
    ];

    let type_identifiers = vec![
        make_node("S", 5, 6),
        make_node("Encoder", 8, 15),
        make_node("T", 17, 18),
        make_node("Encodable", 20, 29),
        make_node("S", 30, 31),
        make_node("Encodable", 34, 43),
        make_node("S", 44, 45),
        make_node("T", 52, 53),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // S: Encoder
    assert_eq!(
        type_variables.get("S").cloned().unwrap_or_default(),
        HashSet::from(["Encoder".into()])
    );

    // T: Encodable<S> — S captured as type arg, making S appear in both its own
    // entry key and in T's bound set
    assert_eq!(
        type_variables.get("T").cloned().unwrap_or_default(),
        HashSet::from(["Encodable".into(), "S".into()])
    );

    assert_eq!(type_variables.len(), 2);

    assert_eq!(
        concrete_types,
        HashSet::from(["Encoder".into(), "Encodable".into(), "S".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_btree_map_encodable() {
    // impl<S: Encoder, K, V> Encodable<S> for BTreeMap<K, V>
    // where
    //     K: Encodable<S> + PartialEq + Ord,
    //     V: Encodable<S>

    let trait_bounds = vec![
        make_node(": Encoder", 6, 15),
        make_node(": Encodable<S> + PartialEq + Ord", 66, 98),
        make_node(": Encodable<S>", 105, 119),
    ];

    let type_identifiers = vec![
        make_node("S", 5, 6),
        make_node("Encoder", 8, 15),
        make_node("K", 17, 18),
        make_node("V", 20, 21),
        make_node("Encodable", 23, 32),
        make_node("S", 33, 34),
        make_node("BTreeMap", 40, 48),
        make_node("K", 49, 50),
        make_node("V", 52, 53),
        make_node("K", 65, 66),
        make_node("Encodable", 68, 77),
        make_node("S", 78, 79),
        make_node("PartialEq", 83, 92),
        make_node("Ord", 95, 98),
        make_node("V", 104, 105),
        make_node("Encodable", 107, 116),
        make_node("S", 117, 118),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // S: Encoder (inline)
    assert_eq!(
        type_variables.get("S").cloned().unwrap_or_default(),
        HashSet::from(["Encoder".into()])
    );

    // K: Encodable<S> + PartialEq + Ord
    // — three-component compound where clause bound, S captured as type arg to Encodable
    assert_eq!(
        type_variables.get("K").cloned().unwrap_or_default(),
        HashSet::from(["Encodable".into(), "S".into(), "PartialEq".into(), "Ord".into()])
    );

    // V: Encodable<S>
    // — same bound trait as K's first component, S again captured as type arg
    assert_eq!(
        type_variables.get("V").cloned().unwrap_or_default(),
        HashSet::from(["Encodable".into(), "S".into()])
    );

    assert_eq!(type_variables.len(), 3);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Encoder".into(),
            "Encodable".into(),
            "S".into(),
            "PartialEq".into(),
            "Ord".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_move_data_builder_fn_ty() {
    // impl<'a, 'tcx, F: Fn(Ty<'tcx>) -> bool> MoveDataBuilder<'a, 'tcx, F>

    let trait_bounds = vec![
        make_node(": Fn(Ty<'tcx>) -> bool", 16, 38),
    ];

    let type_identifiers = vec![
        make_node("F", 15, 16),
        make_node("Fn", 18, 20),
        make_node("Ty", 21, 23),
        make_node("MoveDataBuilder", 40, 55),
        make_node("F", 66, 67),
    ];

    let primitive_types = vec![
        make_node("bool", 34, 38),
    ];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // F: Fn(Ty<'tcx>) -> bool
    // — Fn trait with parenthesised argument syntax: Ty captured as arg type,
    //   'tcx lifetime arg inside Ty<'tcx> invisible,
    //   bool captured as primitive from the return type position
    assert_eq!(
        type_variables.get("F").cloned().unwrap_or_default(),
        HashSet::from(["Fn".into(), "Ty".into(), "bool".into()])
    );

    assert_eq!(type_variables.len(), 1);

    assert_eq!(
        concrete_types,
        HashSet::from(["Fn".into(), "Ty".into(), "bool".into()])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_basic_blocks_no_bounds() {
    // impl<'tcx> BasicBlocks<'tcx>

    let trait_bounds: Option<Vec<NodeMatch>> = None;

    let type_identifiers = vec![
        make_node("BasicBlocks", 11, 22),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, None);

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // No trait bounds at all — impl has only a lifetime parameter 'tcx,
    // which produces no NodeMatch in type_identifiers_flat or primitive_types_flat
    assert!(type_variables.is_empty());
    assert!(concrete_types.is_empty());
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_vec_cache_drop_may_dangle() {
    // unsafe impl<K: Idx, #[may_dangle] V, I> Drop for VecCache<K, V, I>

    let trait_bounds = vec![
        make_node(": Idx", 13, 18),
    ];

    let type_identifiers = vec![
        make_node("K", 12, 13),
        make_node("Idx", 15, 18),
        make_node("V", 34, 35),
        make_node("I", 37, 38),
        make_node("Drop", 40, 44),
        make_node("VecCache", 49, 57),
        make_node("K", 58, 59),
        make_node("V", 61, 62),
        make_node("I", 64, 65),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // K: Idx (inline) — only bounded type param
    assert_eq!(
        type_variables.get("K").cloned().unwrap_or_default(),
        HashSet::from(["Idx".into()])
    );

    // V and I are unbounded (V carries #[may_dangle] attribute, I is plain)
    // — neither produces a type_variables entry
    assert!(type_variables.get("V").is_none());
    assert!(type_variables.get("I").is_none());

    assert_eq!(type_variables.len(), 1);

    assert_eq!(concrete_types, HashSet::from(["Idx".into()]));
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_early_binder_fully_qualified_target() {
    // impl<'s, I: Interner, Iter: IntoIterator> EarlyBinder<I, Iter>
    // where
    //     Iter::Item: Deref,
    //     <Iter::Item as Deref>::Target: Copy + TypeFoldable<I>,

    let trait_bounds = vec![
        make_node(": Interner", 10, 20),
        make_node(": IntoIterator", 26, 40),
        make_node(": Deref", 88, 95),
        make_node(": Copy + TypeFoldable<I>", 131, 155),
    ];

    let type_identifiers = vec![
        make_node("I", 9, 10),
        make_node("Interner", 12, 20),
        make_node("Iter", 22, 26),
        make_node("IntoIterator", 28, 40),
        make_node("EarlyBinder", 42, 53),
        make_node("I", 54, 55),
        make_node("Iter", 57, 61),
        make_node("Item", 84, 88),
        make_node("Deref", 90, 95),
        make_node("Item", 109, 113),
        make_node("Deref", 117, 122),
        make_node("Target", 125, 131),
        make_node("Copy", 133, 137),
        make_node("TypeFoldable", 140, 152),
        make_node("I", 153, 154),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner (inline)
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // Iter: IntoIterator (inline)
    assert_eq!(
        type_variables.get("Iter").cloned().unwrap_or_default(),
        HashSet::from(["IntoIterator".into()])
    );

    // Iter::Item: Deref
    // — simple path projection: final segment "Item" becomes type_var key
    assert_eq!(
        type_variables.get("Item").cloned().unwrap_or_default(),
        HashSet::from(["Deref".into()])
    );

    // <Iter::Item as Deref>::Target: Copy + TypeFoldable<I>
    // — fully-qualified path subject: only the final segment "Target" becomes type_var key,
    //   the entire <Iter::Item as Deref>:: prefix is stripped
    // — compound bound: Copy and TypeFoldable captured, I captured as type arg
    assert_eq!(
        type_variables.get("Target").cloned().unwrap_or_default(),
        HashSet::from(["Copy".into(), "TypeFoldable".into(), "I".into()])
    );

    assert_eq!(type_variables.len(), 4);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Interner".into(),
            "IntoIterator".into(),
            "Deref".into(),
            "Copy".into(),
            "TypeFoldable".into(),
            "I".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_outlives_predicate_lift() {
    // impl<I: Interner, U: Interner, A> Lift<U> for OutlivesPredicate<I, A>
    // where
    //     A: Lift<U>,
    //     I::Region: Lift<U, Lifted = U::Region>,

    let trait_bounds = vec![
        make_node(": Interner", 6, 16),
        make_node(": Interner", 19, 29),
        make_node(": Lift<U>", 81, 90),
        make_node(": Lift<U, Lifted = U::Region>", 105, 134),
    ];

    let type_identifiers = vec![
        make_node("I", 5, 6),
        make_node("Interner", 8, 16),
        make_node("U", 18, 19),
        make_node("Interner", 21, 29),
        make_node("A", 31, 32),
        make_node("Lift", 34, 38),
        make_node("U", 39, 40),
        make_node("OutlivesPredicate", 46, 63),
        make_node("I", 64, 65),
        make_node("A", 67, 68),
        make_node("A", 80, 81),
        make_node("Lift", 83, 87),
        make_node("U", 88, 89),
        make_node("Region", 99, 105),
        make_node("Lift", 107, 111),
        make_node("U", 112, 113),
        make_node("Lifted", 115, 121),
        make_node("Region", 127, 133),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // I: Interner (inline)
    assert_eq!(
        type_variables.get("I").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // U: Interner (inline)
    assert_eq!(
        type_variables.get("U").cloned().unwrap_or_default(),
        HashSet::from(["Interner".into()])
    );

    // A: Lift<U> — type arg U is captured alongside Lift
    assert_eq!(
        type_variables.get("A").cloned().unwrap_or_default(),
        HashSet::from(["Lift".into(), "U".into()])
    );

    // I::Region: Lift<U, Lifted = U::Region>
    // — the path I::Region resolves to type_var "Region" (only final segment matched)
    // — U::Region in the associated type value also contributes only "Region"
    assert_eq!(
        type_variables.get("Region").cloned().unwrap_or_default(),
        HashSet::from(["Lift".into(), "U".into(), "Lifted".into(), "Region".into()])
    );

    assert_eq!(type_variables.len(), 4);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "Interner".into(),
            "Lift".into(),
            "U".into(),
            "Lifted".into(),
            "Region".into(),
        ])
    );
}

    #[test]
fn test_extract_type_identifiers_from_trait_bounds_closure_obligation_processor() {
    // impl<OF, BF, O, E> ObligationProcessor for ClosureObligationProcessor<OF, BF, O, E>
    // where
    //     O: super::ForestObligation + fmt::Debug,
    //     E: fmt::Debug,
    //     OF: FnMut(&mut O) -> ProcessResult<O, E>,
    //     BF: FnMut(&[O]),

    let trait_bounds = vec![
        make_node(": super::ForestObligation + fmt::Debug", 95, 133),
        make_node(": fmt::Debug", 140, 152),
        make_node(": FnMut(&mut O) -> ProcessResult<O, E>", 160, 198),
        make_node(": FnMut(&[O])", 206, 219),
    ];

    let type_identifiers = vec![
        make_node("OF", 5, 7),
        make_node("BF", 9, 11),
        make_node("O", 13, 14),
        make_node("E", 16, 17),
        make_node("ObligationProcessor", 19, 38),
        make_node("ClosureObligationProcessor", 43, 69),
        make_node("OF", 70, 72),
        make_node("BF", 74, 76),
        make_node("O", 78, 79),
        make_node("E", 81, 82),
        make_node("O", 94, 95),
        make_node("ForestObligation", 104, 120),
        make_node("Debug", 128, 133),
        make_node("E", 139, 140),
        make_node("Debug", 147, 152),
        make_node("OF", 158, 160),
        make_node("FnMut", 162, 167),
        make_node("O", 173, 174),
        make_node("ProcessResult", 179, 192),
        make_node("O", 193, 194),
        make_node("E", 196, 197),
        make_node("BF", 204, 206),
        make_node("FnMut", 208, 213),
        make_node("O", 216, 217),
    ];

    let primitive_types: PrimitiveTypeMatches = vec![];

    let ctx: TypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));

    let (type_variables, concrete_types) =
        TypeIdentifiers::extract_type_identifiers_from_trait_bounds(ctx, Some(&trait_bounds));

    let type_variables = type_variables.expect("expected Some(TypeVariableMap)");
    let concrete_types = concrete_types.expect("expected Some(CTypeSet)");

    // O: super::ForestObligation + fmt::Debug
    assert_eq!(
        type_variables.get("O").cloned().unwrap_or_default(),
        HashSet::from(["ForestObligation".into(), "Debug".into()])
    );

    // E: fmt::Debug
    assert_eq!(
        type_variables.get("E").cloned().unwrap_or_default(),
        HashSet::from(["Debug".into()])
    );

    // OF: FnMut(&mut O) -> ProcessResult<O, E>
    assert_eq!(
        type_variables.get("OF").cloned().unwrap_or_default(),
        HashSet::from([
            "FnMut".into(),
            "O".into(),
            "ProcessResult".into(),
            "E".into(),
        ])
    );

    // BF: FnMut(&[O])
    assert_eq!(
        type_variables.get("BF").cloned().unwrap_or_default(),
        HashSet::from(["FnMut".into(), "O".into()])
    );

    assert_eq!(type_variables.len(), 4);

    assert_eq!(
        concrete_types,
        HashSet::from([
            "ForestObligation".into(),
            "Debug".into(),
            "FnMut".into(),
            "ProcessResult".into(),
            "O".into(),
            "E".into(),
        ])
    );
}

#[test]
    fn test_extract_ds_structure_early_binder_with_bracketed_where() {
        // impl<'s, I: Interner, Iter: IntoIterator> EarlyBinder<I, Iter>
        //     where
        //      Iter::Item: Deref,
        //      <Iter::Item as Deref>::Target: Copy + TypeFoldable<I>,
        let type_parameters = vec![make_node("<'s, I: Interner, Iter: IntoIterator>", 4, 41)];
        let type_identifiers = vec![
            make_node("I", 9, 10),
            make_node("Interner", 12, 20),
            make_node("Iter", 22, 26),
            make_node("IntoIterator", 28, 40),
            make_node("EarlyBinder", 42, 53),
            make_node("I", 54, 55),
            make_node("Iter", 57, 61),
            make_node("Item", 84, 88),
            make_node("Deref", 90, 95),
            make_node("Item", 109, 113),
            make_node("Deref", 117, 122),
            make_node("Target", 125, 131),
            make_node("Copy", 133, 137),
            make_node("TypeFoldable", 140, 152),
            make_node("I", 153, 154),
        ];
        let primitive_types: PrimitiveTypeMatches = vec![];
        let where_clause = vec![make_node(
            "where\n     Iter::Item: Deref,\n     <Iter::Item as Deref>::Target: Copy + TypeFoldable<I>,",
            67,
            156,
        )];
        let type_arguments = vec![make_node("<I, Iter>", 53, 62), make_node("<I>", 152, 155)];
        let bracketed_types = vec![make_node("<Iter::Item as Deref>", 102, 123)];
        let ctx: DSTypeCandidatesRef<'_> = (Some(&type_identifiers), Some(&primitive_types));
        let areas_to_avoid: DSAreasToAvoid<'_> = (
            Some(&type_arguments),
            Some(&where_clause),
            Some(&bracketed_types),
            Some(&type_parameters),
        );
        let result = TypeIdentifiers::extract_ds_structure(ctx, None, areas_to_avoid);
        let ds = result.expect("expected Some(NodeMatch) for EarlyBinder");
        assert_eq!(ds.text, "EarlyBinder");
        assert_eq!(ds.range.byte_range.start, 42);
        assert_eq!(ds.range.byte_range.end, 53);
    }
}
