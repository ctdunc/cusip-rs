//! This simple tool reads potential CUSIPs from stdin, one per line, and parses them. It will panic
//! if any fail to parse. This can be used as a simple bulk test of a file of purported CUSIPs to
//! ensure there are no malformed entries present. If you have a known-good file of valid CUSIPs, it
//! can be used to validate this crate considers them valid.
//!
//! As part of the `cusip` crate's initial validation, this tool was run on a file of 1,591,249
//! unique CUSIPs produced by processing a file mapping LEIs to ISINs obtained from GLEIF. The
//! [GLEIF file](https://www.gleif.org/en/lei-data/lei-mapping/download-isin-to-lei-relationship-files)
//! is very large (the version from 2021-02-09 was about 170MB). Here are a few example records for
//! US ISINS:
//!
//! ```sh
//! grep ',US' ISIN_LEI_20210209.csv | head
//! S6XOOCT0IEG5ABCC6L87,US3137A3KN83
//! 254900EDYO1UYWLWP146,US12613N2027
//! 549300DRQQI75D2JP341,US05531GQN42
//! S6XOOCT0IEG5ABCC6L87,US31394GAX16
//! S6XOOCT0IEG5ABCC6L87,US3137ASGH19
//! G5GSEF7VJP5I7OUK5573,US06741RAP64
//! 549300LR1ZETOWYE9Z89,US084601MZ36
//! 8I5DZWZKVSZI1NUHU748,US46636JTK96
//! ANGGYXNX0JLX3X63JN86,US22546ESF24
//! 784F5XWPLTWKTBV3E584,US38143USC61
//! ```
//!
//! You can use a command like this to subset just the US ISINs and convert them to putative CUSIPs:
//!
//! ```sh
//! grep ',US' ISIN_LEI_20210209.csv \
//!   | sed -e 's/^.*,US//' \
//!   | sed -e 's/.$//' \
//!   | sort | uniq | gzip -9 \
//!   > cusips-us.txt.gz
//! ```
//!
//! This file was still about 4.2MB for the version tested.
//!
//! Having produced the file, it is now possible to run it through this tool. From the source
//! directory of this crate, you can run:
//!
//! ```sh
//! gzcat cusips-us.txt.gz | cargo run cusip-tool
//! ```
//!
//! And, output will be something like this:
//!
//! ```text
//! Read 1591249 values; 1591249 were valid CUSIPs and 0 were not.
//! ```
//!
//! If no bad values were found, the tool will exit with zero status, else non-zero.
//!
//! ## Fix mode
//!
//! If you run with argument `--fix`, then any input CUSIPs that are only wrong due to incorrect
//! _Check Digit_ will be fixed. In this mode, every good and every fixable input CUSIP is printed
//! to standard output.

use std::env;
use std::io;
use std::io::prelude::*;
use std::str::from_utf8_unchecked;

#[doc(hidden)]
fn main() {
    let mut fix: bool = false;

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "--fix" {
        fix = true;
    } else if args.len() != 1 {
        eprintln!("usage: cusip-tool [--fix]");
        std::process::exit(1);
    }

    let mut good = 0u64;
    let mut bad = 0u64;
    let mut fixed = 0u64;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        match cusip::parse(&line) {
            Ok(cusip) => {
                good += 1;
                if fix {
                    println!("{cusip}");
                }
            }
            Err(cusip::CUSIPError::IncorrectCheckDigit {
                was: _,
                expected: _,
            }) => {
                bad += 1;
                if fix {
                    let payload = &line.as_bytes()[0..8]; // We know it was the right length
                    let payload = unsafe { from_utf8_unchecked(payload) }; // We know it is ASCII

                    // We know the Check Digit was the only problem, so we can safely unwrap()
                    let cusip = cusip::build_from_payload(payload).unwrap();
                    println!("{cusip}");
                    fixed += 1;
                }
            }
            Err(err) => {
                eprintln!("Input: {line}; Error: {err}");
                bad += 1;
            }
        }
    }

    if fix {
        eprintln!(
            "Read {} values; {} were valid CUSIPs and {} were not. Fixed {}; Omitted {}.",
            good + bad,
            good,
            bad,
            fixed,
            bad - fixed
        );

        if bad > fixed {
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    } else {
        eprintln!(
            "Read {} values; {} were valid CUSIPs and {} were not.",
            good + bad,
            good,
            bad
        );

        let result = (bad == 0) as i32;
        std::process::exit(result);
    }
}
