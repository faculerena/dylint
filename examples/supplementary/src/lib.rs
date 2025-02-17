#![feature(rustc_private)]
#![warn(unused_extern_crates)]

dylint_linting::dylint_library!();

extern crate rustc_lint;
extern crate rustc_session;

#[allow(clippy::no_mangle_with_rust_abi)]
#[no_mangle]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    commented_code::register_lints(sess, lint_store);
    redundant_reference::register_lints(sess, lint_store);
    unnamed_constant::register_lints(sess, lint_store);
    unnecessary_borrow_mut::register_lints(sess, lint_store);
    unnecessary_conversion_for_trait::register_lints(sess, lint_store);
}
