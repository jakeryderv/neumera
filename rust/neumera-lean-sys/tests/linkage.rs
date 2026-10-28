//! Link-time probe: take the address of every symbol in [`REQUIRED_SYMBOLS`].
//! If `libleanshared` is missing any of them the test binary fails to link.

use neumera_lean_sys::REQUIRED_SYMBOLS;

#[test]
#[cfg_attr(
    miri,
    ignore = "asserts link-time symbol resolution; Miri does not link libleanshared"
)]
fn all_required_symbols_resolve_at_link_time() {
    let addresses: &[(*const (), &str)] = &[
        // init / runtime
        (
            neumera_lean_sys::init::lean_initialize as *const (),
            "lean_initialize",
        ),
        (
            neumera_lean_sys::init::lean_initialize_runtime_module as *const (),
            "lean_initialize_runtime_module",
        ),
        (
            neumera_lean_sys::init::lean_initialize_thread as *const (),
            "lean_initialize_thread",
        ),
        (
            neumera_lean_sys::init::lean_finalize_thread as *const (),
            "lean_finalize_thread",
        ),
        (
            neumera_lean_sys::init::lean_setup_args as *const (),
            "lean_setup_args",
        ),
        (
            neumera_lean_sys::init::lean_init_task_manager as *const (),
            "lean_init_task_manager",
        ),
        (
            neumera_lean_sys::init::lean_init_task_manager_using as *const (),
            "lean_init_task_manager_using",
        ),
        (
            neumera_lean_sys::init::lean_finalize_task_manager as *const (),
            "lean_finalize_task_manager",
        ),
        (
            neumera_lean_sys::memory::lean_internal_set_max_memory as *const (),
            "lean_internal_set_max_memory",
        ),
        // refcount + marking
        (
            neumera_lean_sys::refcount::lean_dec_ref_cold as *const (),
            "lean_dec_ref_cold",
        ),
        (
            neumera_lean_sys::refcount::lean_mark_mt as *const (),
            "lean_mark_mt",
        ),
        (
            neumera_lean_sys::refcount::lean_mark_persistent as *const (),
            "lean_mark_persistent",
        ),
        // object / allocators
        (
            neumera_lean_sys::object::lean_alloc_object as *const (),
            "lean_alloc_object",
        ),
        (
            neumera_lean_sys::object::lean_free_object as *const (),
            "lean_free_object",
        ),
        (
            neumera_lean_sys::object::lean_object_byte_size as *const (),
            "lean_object_byte_size",
        ),
        (
            neumera_lean_sys::object::lean_object_data_byte_size as *const (),
            "lean_object_data_byte_size",
        ),
        // arrays
        (
            neumera_lean_sys::array::lean_array_mk as *const (),
            "lean_array_mk",
        ),
        (
            neumera_lean_sys::array::lean_array_push as *const (),
            "lean_array_push",
        ),
        (
            neumera_lean_sys::array::lean_array_to_list as *const (),
            "lean_array_to_list",
        ),
        (
            neumera_lean_sys::array::lean_array_get_panic as *const (),
            "lean_array_get_panic",
        ),
        (
            neumera_lean_sys::array::lean_array_set_panic as *const (),
            "lean_array_set_panic",
        ),
        // strings
        (
            neumera_lean_sys::string::lean_mk_string as *const (),
            "lean_mk_string",
        ),
        (
            neumera_lean_sys::string::lean_mk_string_unchecked as *const (),
            "lean_mk_string_unchecked",
        ),
        (
            neumera_lean_sys::string::lean_mk_string_from_bytes as *const (),
            "lean_mk_string_from_bytes",
        ),
        (
            neumera_lean_sys::string::lean_mk_string_from_bytes_unchecked as *const (),
            "lean_mk_string_from_bytes_unchecked",
        ),
        (
            neumera_lean_sys::string::lean_mk_ascii_string_unchecked as *const (),
            "lean_mk_ascii_string_unchecked",
        ),
        (
            neumera_lean_sys::string::lean_string_push as *const (),
            "lean_string_push",
        ),
        (
            neumera_lean_sys::string::lean_string_append as *const (),
            "lean_string_append",
        ),
        (
            neumera_lean_sys::string::lean_string_mk as *const (),
            "lean_string_mk",
        ),
        (
            neumera_lean_sys::string::lean_string_data as *const (),
            "lean_string_data",
        ),
        (
            neumera_lean_sys::string::lean_string_utf8_get as *const (),
            "lean_string_utf8_get",
        ),
        (
            neumera_lean_sys::string::lean_string_utf8_next as *const (),
            "lean_string_utf8_next",
        ),
        (
            neumera_lean_sys::string::lean_string_utf8_prev as *const (),
            "lean_string_utf8_prev",
        ),
        (
            neumera_lean_sys::string::lean_string_utf8_set as *const (),
            "lean_string_utf8_set",
        ),
        (
            neumera_lean_sys::string::lean_string_utf8_extract as *const (),
            "lean_string_utf8_extract",
        ),
        (
            neumera_lean_sys::string::lean_string_eq_cold as *const (),
            "lean_string_eq_cold",
        ),
        (
            neumera_lean_sys::string::lean_string_lt as *const (),
            "lean_string_lt",
        ),
        (
            neumera_lean_sys::string::lean_string_hash as *const (),
            "lean_string_hash",
        ),
        (
            neumera_lean_sys::string::lean_utf8_strlen as *const (),
            "lean_utf8_strlen",
        ),
        (
            neumera_lean_sys::string::lean_utf8_n_strlen as *const (),
            "lean_utf8_n_strlen",
        ),
        // Nat bignum
        (
            neumera_lean_sys::nat_int::lean_nat_big_succ as *const (),
            "lean_nat_big_succ",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_add as *const (),
            "lean_nat_big_add",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_sub as *const (),
            "lean_nat_big_sub",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_mul as *const (),
            "lean_nat_big_mul",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_div as *const (),
            "lean_nat_big_div",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_mod as *const (),
            "lean_nat_big_mod",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_eq as *const (),
            "lean_nat_big_eq",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_le as *const (),
            "lean_nat_big_le",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_big_lt as *const (),
            "lean_nat_big_lt",
        ),
        (
            neumera_lean_sys::nat_int::lean_nat_overflow_mul as *const (),
            "lean_nat_overflow_mul",
        ),
        // Int bignum
        (
            neumera_lean_sys::nat_int::lean_int_big_neg as *const (),
            "lean_int_big_neg",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_add as *const (),
            "lean_int_big_add",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_sub as *const (),
            "lean_int_big_sub",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_mul as *const (),
            "lean_int_big_mul",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_div as *const (),
            "lean_int_big_div",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_mod as *const (),
            "lean_int_big_mod",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_eq as *const (),
            "lean_int_big_eq",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_le as *const (),
            "lean_int_big_le",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_lt as *const (),
            "lean_int_big_lt",
        ),
        (
            neumera_lean_sys::nat_int::lean_int_big_nonneg as *const (),
            "lean_int_big_nonneg",
        ),
        // Widening
        (
            neumera_lean_sys::nat_int::lean_big_usize_to_nat as *const (),
            "lean_big_usize_to_nat",
        ),
        (
            neumera_lean_sys::nat_int::lean_big_uint64_to_nat as *const (),
            "lean_big_uint64_to_nat",
        ),
        (
            neumera_lean_sys::nat_int::lean_cstr_to_nat as *const (),
            "lean_cstr_to_nat",
        ),
        (
            neumera_lean_sys::nat_int::lean_big_int_to_int as *const (),
            "lean_big_int_to_int",
        ),
        (
            neumera_lean_sys::nat_int::lean_big_size_t_to_int as *const (),
            "lean_big_size_t_to_int",
        ),
        (
            neumera_lean_sys::nat_int::lean_big_int64_to_int as *const (),
            "lean_big_int64_to_int",
        ),
        (
            neumera_lean_sys::nat_int::lean_cstr_to_int as *const (),
            "lean_cstr_to_int",
        ),
        (
            neumera_lean_sys::nat_int::lean_uint8_of_big_nat as *const (),
            "lean_uint8_of_big_nat",
        ),
        // closure dispatch
        (
            neumera_lean_sys::closure::lean_apply_1 as *const (),
            "lean_apply_1",
        ),
        (
            neumera_lean_sys::closure::lean_apply_2 as *const (),
            "lean_apply_2",
        ),
        (
            neumera_lean_sys::closure::lean_apply_3 as *const (),
            "lean_apply_3",
        ),
        (
            neumera_lean_sys::closure::lean_apply_4 as *const (),
            "lean_apply_4",
        ),
        (
            neumera_lean_sys::closure::lean_apply_5 as *const (),
            "lean_apply_5",
        ),
        (
            neumera_lean_sys::closure::lean_apply_6 as *const (),
            "lean_apply_6",
        ),
        (
            neumera_lean_sys::closure::lean_apply_7 as *const (),
            "lean_apply_7",
        ),
        (
            neumera_lean_sys::closure::lean_apply_8 as *const (),
            "lean_apply_8",
        ),
        (
            neumera_lean_sys::closure::lean_apply_9 as *const (),
            "lean_apply_9",
        ),
        (
            neumera_lean_sys::closure::lean_apply_10 as *const (),
            "lean_apply_10",
        ),
        (
            neumera_lean_sys::closure::lean_apply_11 as *const (),
            "lean_apply_11",
        ),
        (
            neumera_lean_sys::closure::lean_apply_12 as *const (),
            "lean_apply_12",
        ),
        (
            neumera_lean_sys::closure::lean_apply_13 as *const (),
            "lean_apply_13",
        ),
        (
            neumera_lean_sys::closure::lean_apply_14 as *const (),
            "lean_apply_14",
        ),
        (
            neumera_lean_sys::closure::lean_apply_15 as *const (),
            "lean_apply_15",
        ),
        (
            neumera_lean_sys::closure::lean_apply_16 as *const (),
            "lean_apply_16",
        ),
        (
            neumera_lean_sys::closure::lean_apply_n as *const (),
            "lean_apply_n",
        ),
        (
            neumera_lean_sys::closure::lean_apply_m as *const (),
            "lean_apply_m",
        ),
        // IO
        (
            neumera_lean_sys::io::lean_io_mark_end_initialization as *const (),
            "lean_io_mark_end_initialization",
        ),
        // external
        (
            neumera_lean_sys::external::lean_register_external_class as *const (),
            "lean_register_external_class",
        ),
    ];

    // Every address must be non-null (failing to link manifests as a build
    // error, but null after relocation is also a regression worth catching).
    for (addr, name) in addresses {
        assert!(!addr.is_null(), "{name} resolved to a null address");
    }

    // The hand-maintained list and the probe list above must stay in sync;
    // catching drift here is cheaper than chasing it through the doc tools.
    assert_eq!(
        addresses.len(),
        REQUIRED_SYMBOLS.len(),
        "tests/linkage.rs probes {} symbols but REQUIRED_SYMBOLS lists {}; \
         keep them in lockstep",
        addresses.len(),
        REQUIRED_SYMBOLS.len(),
    );
}
