// Copyright (c) 2021 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

mod codefall;
mod command;
mod config;
mod handler;

use anyhow::Result;
use log::info;

use handler::CommandHandler;

pub struct Golem {}

impl Golem {
    pub async fn run(&self) -> Result<()> {
        info!("Verifying configuration…");
        config::verify()?;

        let mut cmd_hdl = CommandHandler::new().await?;

        cmd_hdl.run().await
    }
}
