use crate::utils::{match_qpath, snippet, span_help_and_lint, match_def_path};
use rustc::declare_lint_pass;
use rustc::lint::{LateContext, LateLintPass, LintArray, LintPass};
use rustc::ty;
use rustc_hir::{Expr, ExprKind};
use rustc_session::declare_tool_lint;

// TODO: documentation
declare_clippy_lint!(
    pub ATOMIC_ORDERING,
    correctness,
    "Use of an incorrect atomic ordering will cause panic"
);

#[derive(Copy, Clone)]
pub struct AtomicOrdering;

declare_lint_pass!(InvalidAtomicOrdering => [ATOMIC_ORDERING]);

const ATOMIC_ORDERING_ACQUIRE: [&str; 5] = ["core", "sync", "atomic", "Ordering", "Acquire"];
const ATOMIC_ORDERING_ACQREL: [&str; 5] = ["core", "sync", "atomic", "Ordering", "AcqRel"];
const ATOMIC_ORDERING_RELEASE: [&str; 5] = ["core", "sync", "atomic", "Ordering", "Release"];
const ATOMICS: [&str; 12] = [
    "AtomicBool",
    "AtomicI8",
    "AtomicI16",
    "AtomicI32",
    "AtomicI64",
    "AtomicIsize",
    "AtomicPtr",
    "AtomicU8",
    "AtomicU16",
    "AtomicU32",
    "AtomicU64",
    "AtomicUsize",
];

pub fn is_atomic(cx: &LateContext<'_, '_>, expr: &Expr<'_>) -> bool {
    if let ty::Adt(&ty::AdtDef { did, ..}, _) = cx.tables.expr_ty(expr).kind {
        ATOMICS.iter().any(|ty| match_def_path(cx, did, &["core", "sync", "atomic", ty]))
    } else {
        false
    }
}

pub fn is_invalid_store(method_name: rustc_span::symbol::Symbol, order: &ExprKind<'_>) -> bool {
    if method_name == sym!(store) {
        if let ExprKind::Path(ref qp) = order {
            // check for invalid store ordering
            if match_qpath(qp, &ATOMIC_ORDERING_ACQUIRE) || match_qpath(qp, &ATOMIC_ORDERING_ACQREL) {
                return true;
            }
        }
    }
    false
}

pub fn is_invalid_load(method_name: rustc_span::symbol::Symbol, order: &ExprKind<'_>) -> bool {
    if method_name == sym!(load) {
        if let ExprKind::Path(ref qp) = order {
            // check for invalid load ordering
            if match_qpath(qp, &ATOMIC_ORDERING_RELEASE) || match_qpath(qp, &ATOMIC_ORDERING_ACQREL) {
                return true;
            }
        }
    }
    false
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for InvalidAtomicOrdering {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, e: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(ref path, _, ref args) = e.kind {
            if is_atomic(cx, &args[0])
                && (is_invalid_store(path.ident.name, &args[2].kind) || is_invalid_load(path.ident.name, &args[1].kind))
            {
                let ordering = if path.ident.name == sym!(load) {
                    ["SeqCst", "Acquire", "Relaxed"]
                } else {
                    ["SeqCst", "Release", "Relaxed"]
                };
                span_help_and_lint(
                    cx,
                    ATOMIC_ORDERING,
                    args[2].span,
                    &format!(
                        "`{}` is not a valid `atomic::Ordering` for `{}`",
                        snippet(cx, args[2].span, ".."),
                        path.ident.name
                    ),
                    &format!("valid orderings are {:?}", ordering),
                );
            }
        }
    }
}
