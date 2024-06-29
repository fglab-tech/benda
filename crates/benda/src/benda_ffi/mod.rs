use bend::diagnostics::{Diagnostics, DiagnosticsConfig};
use bend::fun::{Book, Term};
use bend::{CompileOpts, RunOpts};

pub fn run(book: &Book) -> Option<(Term, String, Diagnostics)> {
    let run_opts = RunOpts::default();
    let compile_opts = CompileOpts::default().set_all();
    let diagnostics_cfg = DiagnosticsConfig::default();
    let args = None;

    bend::run_book(
        book.to_owned(),
        run_opts,
        compile_opts,
        diagnostics_cfg,
        args,
        "run",
    )
    .unwrap()
}
