use rustc::lint::*;
use rustc::hir;
use rustc::hir::def_id::DefId;

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
        let def_id = cx.tcx.hir.local_def_id(item.id);
        match item.node {
            hir::ItemTrait(..) =>
                check_new_trait(cx, def_id),
            hir::ItemEnum(..) | hir::ItemStruct(..) | hir::ItemUnion(..) =>
                check_new_type(cx, def_id),
            _ => {}
        }
    }
}

fn check_new_trait(cx: &LateContext, trait_def_id: DefId) {
    let trait_impls = cx.tcx.hir.trait_impls(trait_def_id);
    for &trait_impl_id in trait_impls {
        eprintln!("{:?}", cx.tcx.impl_trait_ref(cx.tcx.hir.local_def_id(trait_impl_id)));
    }
}

fn check_new_type(cx: &LateContext, def_id: DefId) {}
