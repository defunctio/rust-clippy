use crate::utils::{is_copy, is_self_ty, snippet, span_lint_and_sugg};
use matches::matches;
use rustc::hir;
use rustc::hir::intravisit::FnKind;
use rustc::hir::*;
use rustc::impl_lint_pass;
use rustc::lint::{LateContext, LateLintPass, LintArray, LintPass};
use rustc::session::config::Config as SessionConfig;
use rustc::ty;
use rustc_errors::Applicability;
use rustc_session::declare_tool_lint;
use rustc_span::Span;
use rustc_target::abi::LayoutOf;
use rustc_target::spec::abi::Abi;

declare_clippy_lint! {
    /// **What it does:** Checks for functions taking arguments by value, where
    /// the argument type is `Copy` and large enough to be more efficient to always
    /// pass by ref.
    ///
    /// **Why is this bad?** Passing large immutable structures by value requires
    /// copying of underlying data needlessly resulting in performance degradation.
    ///
    /// **Known problems:** This lint is target register size dependent, it is
    /// limited to 32-bit to try and reduce portability problems between 32 and
    /// 64-bit, but if you are compiling for 8 or 16-bit targets then the limit
    /// will be different.
    ///
    /// The configuration option `large_data_size_min_limit` can be set to override
    /// this limit for a project.
    ///
    ///
    /// **Example:**
    ///
    /// ```rust
    /// // Bad
    /// fn foo(v: BigStruct) {}
    /// ```
    ///
    /// ```rust
    /// // Better
    /// fn foo(v: &BigStruct) {}
    /// ```
    pub LARGE_DATA_PASS_BY_VAL,
    perf,
    "functions taking large copyable arguments by value"
}

#[derive(Copy, Clone)]
pub struct LargeDataPassByVal {
    limit: u64,
}

impl<'a, 'tcx> LargeDataPassByVal {
    pub fn new(limit: Option<u64>, target: &SessionConfig) -> Self {
        #[allow(clippy::integer_division)]
        Self {
            limit: limit.unwrap_or((u64::from(target.ptr_width) / 8) * 2),
        }
    }

    fn check_poly_fn(&mut self, cx: &LateContext<'_, 'tcx>, hir_id: HirId, decl: &FnDecl<'_>, span: Option<Span>) {
        let fn_def_id = cx.tcx.hir().local_def_id(hir_id);

        let fn_sig = cx.tcx.fn_sig(fn_def_id);
        let fn_sig = cx.tcx.erase_late_bound_regions(&fn_sig);

        for (input, &ty) in decl.inputs.iter().zip(fn_sig.inputs()) {
            // All spans generated from a proc-macro invocation are the same...
            match span {
                Some(s) if s == input.span => return,
                _ => (),
            }
            if let ty::Adt(..) = ty.kind {
                if is_copy(cx, ty) {
                    if let Some(size) = cx.layout_of(ty).ok().map(|l| l.size.bytes()) {
                        if size > self.limit {
                            let value_type = if is_self_ty(input) {
                                "self".into()
                            } else {
                                snippet(cx, input.span, "_").into()
                            };
                            span_lint_and_sugg(
                                cx,
                                LARGE_DATA_PASS_BY_VAL,
                                input.span,
                                &format!("this argument ({} byte) is passed by value, but would be more efficient if passed by ref (limit: {} byte)", size, self.limit),
                                "consider passing by ref instead",
                                value_type,
                                Applicability::Unspecified,
                            );
                        }
                    }
                }
            }
        }
    }
}

impl_lint_pass!(LargeDataPassByVal => [LARGE_DATA_PASS_BY_VAL]);

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for LargeDataPassByVal {
    fn check_trait_item(&mut self, cx: &LateContext<'a, 'tcx>, item: &'tcx hir::TraitItem<'_>) {
        if item.span.from_expansion() {
            return;
        }

        if let hir::TraitItemKind::Method(method_sig, _) = &item.kind {
            self.check_poly_fn(cx, item.hir_id, &*method_sig.decl, None);
        }
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        kind: FnKind<'tcx>,
        decl: &'tcx FnDecl<'_>,
        _body: &'tcx Body<'_>,
        span: Span,
        hir_id: HirId,
    ) {
        if span.from_expansion() {
            return;
        }

        match kind {
            FnKind::ItemFn(.., header, _, attrs) => {
                if header.abi != Abi::Rust {
                    return;
                }
                for a in attrs {
                    if a.meta_item_list().is_some() && a.check_name(sym!(proc_macro_derive)) {
                        return;
                    }
                }
            },
            FnKind::Method(..) => (),
            _ => return,
        }

        // Exclude non-inherent impls
        if let Some(Node::Item(item)) = cx.tcx.hir().find(cx.tcx.hir().get_parent_node(hir_id)) {
            if matches!(item.kind, ItemKind::Impl(_, _, _, _, Some(_), _, _) |
                ItemKind::Trait(..))
            {
                return;
            }
        }

        self.check_poly_fn(cx, hir_id, decl, Some(span));
    }
}
