use rustc::lint::*;

/// **What it does:** TODO
///
/// **Why is this bad?** TODO
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// TODO
/// ```
declare_lint! {
    pub COW_NEEDS_IMPL, Allow,
    "TODO"
}

pub struct CowNeedsImpl;

impl LintPass for CowNeedsImpl {
    fn get_lints(&self) -> LintArray {
        lint_array!(COW_NEEDS_IMPL)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for CowNeedsImpl {
    fn check_item(&mut self, cx: &LateContext<'a, 'tcx>, item: &'tcx hir::Item) {
    }
}
