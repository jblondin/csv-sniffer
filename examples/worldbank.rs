extern crate csv_sniffer;
extern crate csv;

use std::path::Path;

use csv_sniffer::metadata::*;
use csv::{Terminator};

fn main() {
    let data_filepath = Path::new(file!()).parent().unwrap().join("../tests/data/gdp.csv");
    let dialect = Dialect {
        delimiter: b',',
        header: Header { has_header_row: true, num_preamble_rows: 4 },
        quote: Quote::Some { character: b'"', doublequote_escapes: true },
        comment: Comment::Disabled,
        escape: Escape::Disabled,
        terminator: Terminator::CRLF,
        flexible: false,
    };
    let mut reader = dialect.open_path(data_filepath).unwrap();
    for result in reader.records() {
        let record = result.unwrap();
        println!("{:?}", record);
    }
}
