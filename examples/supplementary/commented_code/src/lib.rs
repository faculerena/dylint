#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint_and_help, source::get_source_text};
use if_chain::if_chain;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rustc_hir::Block;
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{BytePos, Span};

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// Checks for code that has been commented out.
    ///
    /// ### Why is this bad?
    /// Commented code is often meant to be removed, but kept by mistake.
    ///
    /// ### Known problems
    /// - Currently only checks for commented out statements in blocks.
    /// - Does not handle statements spanning multiple line comments, e.g.:
    ///
    ///   ```rust
    ///   // dbg!(
    ///   //   x
    ///   // );
    ///   ```
    ///
    /// ### Example
    /// ```rust
    /// # fn f(_: u32) {}
    /// # let x = 0;
    /// // dbg!(x);
    /// f(x);
    /// ```
    /// Use instead:
    /// ```rust
    /// # fn f(_: u32) {}
    /// # let x = 0;
    /// f(x);
    /// ```
    pub COMMENTED_CODE,
    Warn,
    "code that has been commented out"
}

impl<'tcx> LateLintPass<'tcx> for CommentedCode {
    fn check_block(&mut self, cx: &LateContext<'tcx>, block: &'tcx Block<'tcx>) {
        if block.stmts.is_empty() {
            check_span(
                cx,
                block
                    .span
                    .with_lo(block.span.lo() + BytePos(1))
                    .with_hi(block.span.hi() - BytePos(1)),
            );
        } else {
            check_span(
                cx,
                block
                    .span
                    .with_lo(block.span.lo() + BytePos(1))
                    .with_hi(block.stmts.first().unwrap().span.lo()),
            );
            for window in block.stmts.windows(2) {
                check_span(
                    cx,
                    block
                        .span
                        .with_lo(window[0].span.hi())
                        .with_hi(window[1].span.lo()),
                );
            }
            check_span(
                cx,
                block
                    .span
                    .with_lo(block.stmts.last().unwrap().span.hi())
                    .with_hi(block.span.hi() - BytePos(1)),
            );
        }
    }
}

static LINE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new("(^|[^/])(//([^/].*))").unwrap());
static BLOCK_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"/\*(([^*]|\*[^/])*)\*/").unwrap());

fn check_span(cx: &LateContext<'_>, span: Span) {
    let Some(source_file_range) = get_source_text(cx, span) else {
        return;
    };
    let Some(text) = source_file_range.as_str() else {
        return;
    };
    for captures in LINE_COMMENT.captures_iter(text) {
        assert_eq!(4, captures.len());
        check_captures(cx, span, &captures, 2, 3);
    }
    for captures in BLOCK_COMMENT.captures_iter(text) {
        assert_eq!(3, captures.len());
        check_captures(cx, span, &captures, 0, 1);
    }
}

fn check_captures(
    cx: &LateContext<'_>,
    span: Span,
    captures: &Captures,
    span_index: usize,
    text_index: usize,
) {
    let range = captures.get(span_index).unwrap().range();
    let text = &captures[text_index];
    if_chain! {
        if let Ok(block) = syn::parse_str::<syn::Block>(&format!("{{{text}}}"));
        if !block.stmts.is_empty();
        then {
            #[allow(clippy::cast_possible_truncation)]
            span_lint_and_help(
                cx,
                COMMENTED_CODE,
                span.with_lo(span.lo() + BytePos(range.start as u32))
                    .with_hi(span.lo() + BytePos(range.end as u32)),
                "commented out code",
                None,
                "uncomment or remove",
            );
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
