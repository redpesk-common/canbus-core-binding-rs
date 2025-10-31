/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/
extern crate dbcparser;
use dbcparser::prelude::*;

fn main() {
    // Déclare "afbv4" comme cfg connu → plus de warning
    println!("cargo::rustc-check-cfg=cfg(afbv4)");

    // (facultatif) active réellement le cfg si AFBV4=1
    if std::env::var("AFBV4").is_ok() {
        println!("cargo:rustc-cfg=afbv4");
    }

    // let dir = concat!(env!("CARGO_TARGET_DIR", "debug"));
    // println!("cargo:rustc-link-search={}", dir);
    // println!("cargo:rustc-link-lib=liblibafb");

    // !!! WARNING: model3can.dbc generate 33000 lines of code !!!
    //let dbc_infile="../dbc-log/simple.dbc";
    let dbc_infile = "./etc/dbc/model3can.dbc";

    // generate parser outside of project git repo
    let dbc_outfile = "./src/__model3-dbcgen.rs";

    // invalidate build when dbc file changes
    println!("cargo:rerun-if-changed={}", dbc_infile);

    let header = "
// -----------------------------------------------------------------------
//              <- DBC file Rust mapping ->
// -----------------------------------------------------------------------
//  Do not exit this file it will be regenerated automatically by cargo.
//  Check:
//   - build.rs at project root for dynamically mapping
//   - example/demo/dbc-log/??? for static values
//  Reference: iot.bzh/Redpesk canbus-rs code generator
// -----------------------------------------------------------------------
// Tell rustfmt (stable) to skip formatting this whole file
#[rustfmt::skip]

#[allow(
    warnings,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::redundant_field_names,
    clippy::similar_names
)]
    ";
    DbcParser::new("DbcModel3")
        .dbcfile(dbc_infile)
        .outfile(dbc_outfile)
        .header(header)
        .range_check(true)
        .serde_json(true)
        .whitelist(vec![280, 599, 614]) // restrict generated code size to candump.log messages
        .generate()
        .expect("Fail to parse dbc-file'\n");
}
