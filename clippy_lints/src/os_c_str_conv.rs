use utils::match_type;

use rustc::lint::*;
use rustc::hir::*;
use rustc::hir::def_id::DefId;
use rustc::ty;
use syntax::ast::NodeId;
use syntax_pos::Span;

use itertools::zip;

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
    Warn,
    "Lint unnecessary conversions from Os/CStr(ing) to str/String"
}

// ways of getting a str(ing) that could also be os/cstr(ing):
// - to_str/to_string_lossy/into_string
// - std::env::args/vars/var

// usages where conv is 'unecessary':
// - passed only to generic functions where the type parameter that pertains to it does not pertain
// to anything else AND the unconv'd type satisfies all the traits

// cbs: could-be-OS/CStr(ing)

pub struct OsCStrConv {
    cbs_locals: Vec<NodeId>,
}

impl OsCStrConv {
    pub fn new() -> Self {
        OsCStrConv { cbs_locals: Vec::new(), }
    }
}

impl LintPass for OsCStrConv {
    fn get_lints(&self) -> LintArray {
        lint_array!(OS_C_STR_CONV)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for OsCStrConv {
    fn check_pat(&mut self, cx: &LateContext, pat: &Pat) {
        if let PatKind::Binding(_, pat_node_id, _, _) = pat.node {
            eprintln!("checking pat: {:?}", pat);
            let pat_ty = cx.tables.pat_ty(pat);
            eprintln!("  pat_ty: {:?}", pat_ty);
            if (match_type(cx, pat_ty, &["std", "env", "Args"])) {
                // FIXME: data flow analysis?
                eprintln!("  FOUND ARGS!!!");
            }
            // if local could be cstr, record
        }
    }

    fn check_expr(&mut self, cx: &LateContext, expr: &Expr) {
        match expr.node {
            ExprCall(ref fun, ref args) => {
                eprintln!("checking cl: {:?}", expr);
                // only functions from actual paths can be generic
                if let ExprPath(ref qpath) = fun.node {
                    let fn_def = cx.tables.qpath_def(qpath, fun.hir_id);
                    match fn_def {
                        def::Def::Fn(did) | def::Def::Method(did) => {
                            visit_call(cx, did, fun.hir_id, args, expr.span);
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
                let tdd_tbl = cx.tables.type_dependent_defs();
                let opt_method_def = tdd_tbl.get(expr.hir_id);
                if let Some(&def::Def::Method(did)) = opt_method_def {
                    visit_call(cx, did, expr.hir_id, args, expr.span);
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
    // check if expr is a local that has been marked
    if_chain!{
        if let ExprPath(QPath::Resolved(_, ref path)) = e.node;
        if let Some(ref seg) = path.segments.last();
        if seg.name.as_str() == "_cabt_o";
        then {
            // eprintln!("   cbs: {:?}", e);
            true
        } else {
            false
        }
    }
}

// FIXME: remove expr_id, checking_span if necessary
fn visit_call(cx: &LateContext, fn_did: DefId, _expr_id: HirId, args: &[Expr], _checking_span: Span) {
    let args_cbs: Vec<_> = args.iter().map(could_expr_be_str).collect();
    // if !args_cbs.iter().any(|b| *b) { return; }

    // let fn_ty = cx.tcx.type_of(fn_did);
    let fn_sig = cx.tcx.fn_sig(fn_did);
    let fn_generics = cx.tcx.generics_of(fn_did);
    // eprintln!("  ty: {:?}", fn_ty);
    // eprintln!("  sig: {:?}", fn_sig);
    // eprintln!("  generics: {:?}", fn_generics);
    // must be generic if str can be replaced
    // if let Some(ex_substs) = cx.tables.node_substs_opt(expr_id) {
    //     eprintln!("  ex_substs: {:?}", ex_substs);
    //     if let ty::TyFnDef(ty_fn_did, fn_substs) = fn_ty.sty {
    //         if ty_fn_did == fn_did {
    //             eprintln!("  fn_substs: {:?}", fn_substs);
                fn_sig.inputs().map_bound(|fn_inputs| {
                    // eprintln!("  fn_inputs: {:?}", fn_inputs);
                    for fn_input_ty in zip(fn_inputs, args_cbs)
                        .filter(|&(_, cbs)| cbs)
                        .map(|(in_ty, _)| in_ty)
                    {
                        // for each parameter that could be a os/cstr
                        if let ty::TyParam(ref pt) = fn_input_ty.sty {
                            let arg_typaramdef = fn_generics.type_param(pt, cx.tcx);
                            // eprintln!("  atpd: {:?}", arg_typaramdef);
                            let typaram_usage_cnt = fn_inputs.iter()
                                .filter(|&fity| fity.is_param(arg_typaramdef.index)).count();
                            // eprintln!("  usage count: {:?}", typaram_usage_cnt);
                            if typaram_usage_cnt == 1 {
                                // FIXME: return type?
                                // THIS USAGE IS OK FOR NOT CONVERTING
                            } else {
                                // THIS USAGE CANNOT BE REPLACED
                            }
                        }
                    }
                });
    //         } else {
    //             cx.sess().diagnostic().span_bug_no_panic(
    //                 checking_span,
    //                 "`TyFnDef`'s `DefId` did not match the `DefId` it came from",
    //             );
    //         }
    //     } else {
    //         cx.sess().diagnostic().span_bug_no_panic(
    //             checking_span,
    //             "type of `Def::Fn` or `Def::Method` was not `TyFnDef`",
    //         );
    //     }
    // }
}
