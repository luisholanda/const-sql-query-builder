//! # A pure const SQL query builder
//!
//! This crate has the objetive of experimenting with unstable `const fn`
//! features to create a fully const query builder.
#![feature(
    adt_const_params,
    allocator_api,
    core_intrinsics,
    const_mut_refs,
    const_eval_select,
    const_box,
    const_heap,
    const_ptr_is_null,
    const_ptr_write,
    const_slice_from_raw_parts,
    const_option_ext,
    const_alloc_error,
    const_nonnull_new,
    const_trait_impl,
    const_precise_live_drops,
    const_swap,
    generic_const_exprs,
    marker_trait_attr
)]

pub(crate) mod const_string;
pub mod expression;
pub mod query;
pub mod schema;
