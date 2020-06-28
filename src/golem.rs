// Copyright (c) 2020 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

mod codefall;
mod command;
mod config;
mod handler;

use anyhow::Result;
use log::{debug, info};

use handler::CommandHandler;

pub struct Golem {}

impl Golem {
    pub async fn run(&self) -> Result<()> {
        info!("Verifying configuration…");
        config::verify()?;

        let mut cmd_hdl = CommandHandler::new().await?;

        debug!("Created command handler: {:#?}", cmd_hdl);

        cmd_hdl.run().await
    }
}
