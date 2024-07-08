//! Bend Interface Module
//!
//! This module provides functionality for running Bend books and commands.
//! It includes a `run` function that executes a command on a given Bend book
//! and returns the result along with diagnostics.
//!
//! # Dependencies
//!
//! - `bend::diagnostics`: For handling diagnostics during execution
//! - `bend::fun`: For working with Bend books and terms
//! - `bend`: For compilation and runtime options

use bend::diagnostics::{Diagnostics, DiagnosticsConfig};
use bend::fun::{Book, Term};
use bend::{CompileOpts, RunOpts};

/**
 Runs a command on a book and returns the result.

 # Arguments

 * `book` - The book to run in the HVM.
 * `cmd` - The runtime to run the book on: Rust, C or CUDA.

 # Returns

 An `Option` containing a tuple with the following elements:
 * `Term` - The resulting term.
 * `String` - The output of the command.
 * `Diagnostics` - Any diagnostics generated during the execution.

 # Panics

 This function will panic if the HVM execution fails.
*/
pub fn run(
    book: &Book,
    cmd: &str,
) -> Result<Option<(Term, String, Diagnostics)>, Diagnostics> {
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
        cmd,
    )
}
