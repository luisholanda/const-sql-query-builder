//! # A pure const SQL query builder
//!
//! This crate has the objetive of experimenting with unstable `const fn`
//! features to create a fully const query builder.
#![allow(incomplete_features)]
#![feature(
    adt_const_params,
    allocator_api,
    const_alloc_error,
    const_alloc_layout,
    const_box,
    const_eval_select,
    const_fmt_arguments_new,
    const_heap,
    const_maybe_uninit_uninit_array,
    const_maybe_uninit_write,
    const_mut_refs,
    const_nonnull_new,
    const_option,
    const_option_ext,
    const_precise_live_drops,
    const_ptr_as_ref,
    const_ptr_is_null,
    const_ptr_read,
    const_ptr_write,
    const_refs_to_cell,
    const_slice_from_raw_parts_mut,
    const_swap,
    const_trait_impl,
    core_intrinsics,
    generic_const_exprs,
    macro_metavar_expr,
    marker_trait_attr,
    maybe_uninit_uninit_array,
    slice_ptr_get,
)]

pub(crate) mod const_string;
pub mod expression;
pub mod query;
pub mod schema;
