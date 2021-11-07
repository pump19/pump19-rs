// Copyright (c) 2021 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#![recursion_limit = "256"]

mod golem;

use anyhow::Result;

use golem::Golem;

#[tokio::main]
async fn main() -> Result<()> {
    if cfg!(debug_assertions) {
        dotenv::dotenv()?;
    }

    pretty_env_logger::init();

    log::info!("Running pump19 rust golem…");

    Golem {}.run().await
}
