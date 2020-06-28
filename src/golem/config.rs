// Copyright (c) 2020 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::env;

use anyhow::{format_err, Result};
use log::debug;

const CONFIG_VARS: &[&str] = &[
    "PUMP19_TRIGGERS",
    "PUMP19_IRC_NICKNAME",
    "PUMP19_IRC_USERNAME",
    "PUMP19_IRC_PASSWORD",
    "PUMP19_IRC_CHANNELS",
    "PUMP19_CODEFALL_DATABASE",
    "PUMP19_CODEFALL_CHANNELS",
    "PUMP19_URL_HELP",
    "PUMP19_URL_BINGO",
    "PUMP19_URL_CODEFALL",
];

pub fn verify() -> Result<()> {
    for var in CONFIG_VARS {
        debug!(
            "{} = {}",
            var,
            env::var(var).map_err(|e| format_err!("{}: {}", e, var))?
        );
    }

    Ok(())
}
