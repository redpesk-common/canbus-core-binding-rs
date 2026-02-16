/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/

// Build script to generate Rust bindings DBC file using `dbcparser`.
// It configures an optional `afbv4` cfg flag and emits a generated source file under `src/`.

// Import the code generator entry points used to transform a DBC file into Rust structures.
use dbcparser::gencode::DbcParser;
use dbcparser::gencode::DEFAULT_HEADER;

// Build script configuring a conditional compilation flag for the `afbv4` feature.
//
// This script:
// - declares the custom `afbv4` cfg so the compiler accepts it,
// - enables the `afbv4` configuration when the `AFBV4` environment variable is set.
// - generates Rust code from a DBC file using the `dbcparser` crate.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inform rustc that `cfg(afbv4)` is a valid configuration key to avoid unknown cfg warnings.
    println!("cargo::rustc-check-cfg=cfg(afbv4)");

    // When the `AFBV4` environment variable is present, propagate a cfg flag to the crate
    // so that code can use `#[cfg(afbv4)]` sections.
    if std::env::var("AFBV4").is_ok() {
        println!("cargo:rustc-cfg=afbv4");
    }

    let dbc_infile = "../samples/bms/dbc/BMS.dbc";

    // generate parser outside of project git repo
    let dbc_outfile = "./src/__bms-dbcgen.rs";

    // invalidate build when dbc file changes
    println!("cargo:rerun-if-changed={}", dbc_infile);

    DbcParser::new("DbcBms")
        .dbcfile(dbc_infile)
        .outfile(dbc_outfile)
        .header(DEFAULT_HEADER)
        .range_check(true)
        .serde_json(true)
        .whitelist(vec![257, 545, 641]) // restrict generated code size to candump.log messages
        .generate()?;
    Ok(())
}
