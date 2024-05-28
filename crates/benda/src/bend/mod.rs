use std::path::Path;

use bend::{diagnostics::{self, DiagnosticsConfig}, fun::Book, CompileOpts, RunOpts};

pub fn run(book: &Book ) {
    let run_opts = RunOpts { linear_readback: false, pretty: false };
    let compile_opts = CompileOpts::default();
    let diagnostics_cfg = DiagnosticsConfig::default();
    let args = None;

    let result = bend::run_book(book.clone(), run_opts, compile_opts, diagnostics_cfg, args, "run-cu").unwrap();

    println!("{:?}", result);

}