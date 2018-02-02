use rustc::lint::*;
use rustc::hir::*;
use rustc::hir::def_id::DefId;
use rustc::ty;
use syntax_pos::Span;

/// **What it does:** Checks for converting an `OsStr(ing)` or `CStr(ing)` to a `str`/`String`,
/// then immediately using it where the original type would have worked.
///
/// **Why is this bad?** The conversion is unnecessary.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// let files = Vec::new();
/// for arg in env::args() {
///     files.push(File::open(arg));
/// }
/// ```
///
/// Could be written:
///
/// ```rust
/// let files = Vec::new();
/// for arg in env::args_os() {
///     files.push(File::open(arg));
/// }
/// ```
declare_lint! {
    pub OS_C_STR_CONV,
    Allow,
    "Lint unnecessary conversions from Os/CStr(ing) to str/String"
}

pub struct OsCStrConv;

impl LintPass for OsCStrConv {
    fn get_lints(&self) -> LintArray {
        lint_array!(OS_C_STR_CONV)
    }
}

// ways of getting a str(ing) that could also be os/cstr(ing):
// - to_str/to_string_lossy/into_string
// - std::env::args/vars/var

// usages where conv is 'unecessary':
// - passed only to generic functions where the type parameter that pertains to it does not pertain
// to anything else AND the unconv'd type satisfies all the traits

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for OsCStrConv {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        match expr.node {
            ExprCall(ref fun, ref args) => {
                eprintln!("checking cl: {:?}", expr);
                eprintln!(
                    "  args: {:?}",
                    args.iter().map(could_expr_be_str).collect::<Vec<_>>()
                );
                // only functions from actual paths can be generic
                // so only those can have its argument types changed from str to osstr
                if let ExprPath(ref qpath) = fun.node {
                    let fn_def = cx.tables.qpath_def(qpath, fun.hir_id);
                    match fn_def {
                        def::Def::Fn(did) | def::Def::Method(did) => {
                            do_lint(cx, did, fun.hir_id, expr.span);
                        },
                        _ => {
                            eprintln!("  error_def: {:?}", fn_def);
                            cx.sess().diagnostic().span_bug_no_panic(
                                fun.span,
                                "`ExprCall` of a path did not resolve to a `Def::Fn` or `Def::Method`",
                            );
                        },
                    }
                }
            },
            ExprMethodCall(_, _, ref args) => {
                eprintln!("checking md: {:?}", expr);
                args.iter()
                    .enumerate()
                    .filter(|e| could_expr_be_str(e.1))
                    .map(|e| e.0)
                    .collect::<Vec<_>>();
                let tdd_tbl = cx.tables.type_dependent_defs();
                let opt_method_def = tdd_tbl.get(expr.hir_id);
                if let Some(&def::Def::Method(did)) = opt_method_def {
                    do_lint(cx, did, expr.hir_id, expr.span);
                } else {
                    eprintln!("  error_def: {:?}", opt_method_def);
                    cx.sess().diagnostic().span_bug_no_panic(
                        expr.span,
                        "`ExprMethodCall` did not resolve to a `Def::Method`",
                    );
                }
            },
            _ => (),
        }
    }
}

fn could_expr_be_str(e: &Expr) -> bool {
    // FIXME
    if_chain!{
        if let ExprPath(QPath::Resolved(_, ref path)) = e.node;
        if let Some(ref seg) = path.segments.last();
        if seg.name.as_str() == "_cabt_o";
        then {
            eprintln!("   cbs: {:?}", e);
            true
        } else {
            false
        }
    }
}

// FIXME: rename this
fn do_lint(cx: &LateContext, fn_did: DefId, expr_id: HirId, checking_span: Span) {
    let fn_ty = cx.tcx.type_of(fn_did);
    let fn_sig = cx.tcx.fn_sig(fn_did);
    eprintln!("  sig: {:?}", fn_sig);
    eprintln!("  ty: {:?}", fn_ty);
    let opt_substs = cx.tables.node_substs_opt(expr_id);
    if let Some(ex_substs) = opt_substs {
        eprintln!("  ex_substs: {:?}", ex_substs);
        if let ty::TyFnDef(ty_did, ty_substs) = fn_ty.sty {
            if ty_did == fn_did {
                eprintln!("  ty_substs: {:?}", ty_substs);
            } else {
                cx.sess().diagnostic().span_bug_no_panic(
                    checking_span,
                    "`TyFnDef`'s `DefId` did not match the `DefId` it came from",
                );
            }
        } else {
            cx.sess().diagnostic().span_bug_no_panic(
                checking_span,
                "type of `Def::Fn` or `Def::Method` was not `TyFnDef`",
            );
        }
    }
}
