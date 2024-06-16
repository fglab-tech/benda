mod parser;

use std::path::{self, Path};

use pyo3::prelude::*;
use rustpython_parser::{parse, Mode};
use types::book::Book;

mod benda_ffi;
mod types;

fn main() {
    let new_path = Path::new("./examples/quicksort.bend");
    let bend_book = bend::load_file_to_book(new_path);

    //let code = std::fs::read_to_string(new_path)
    //    .map_err(|e| e.to_string())
    //    .unwrap();
    //let bend_book = bend::fun::load_book::do_parse_book(
    //    &code,
    //    new_path,
    //    BendBook::default(),
    //);

    let book = Book::new(bend_book.unwrap());
}
