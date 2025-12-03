/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/

use dbcparser::gencode::DbcParser;
use dbcparser::gencode::DEFAULT_HEADER;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Déclare "afbv4" comme cfg connu → plus de warning
    println!("cargo::rustc-check-cfg=cfg(afbv4)");

    // (facultatif) active réellement le cfg si AFBV4=1
    if std::env::var("AFBV4").is_ok() {
        println!("cargo:rustc-cfg=afbv4");
    }

    let dbc_infile = "../samples/model3/dbc/model3can.dbc";

    // generate parser outside of project git repo
    let dbc_outfile = "./src/__model3-dbcgen.rs";

    // invalidate build when dbc file changes
    println!("cargo:rerun-if-changed={}", dbc_infile);

    DbcParser::new("DbcModel3")
        .dbcfile(dbc_infile)
        .outfile(dbc_outfile)
        .header(DEFAULT_HEADER)
        .range_check(true)
        .serde_json(true)
        .whitelist(vec![280, 599, 614]) // restrict generated code size to candump.log messages
        .generate()?;
    Ok(())
}
