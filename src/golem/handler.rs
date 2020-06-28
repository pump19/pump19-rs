// Copyright (c) 2020 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::env;

use anyhow::Result;
use futures::{stream, StreamExt};
use irc::client::prelude::{
    Client as IrcClient, Command as IrcCommand, Config as IrcConfig, Message as IrcMessage, Prefix,
};
use log::{debug, error, info};
use regex::Regex;

use super::codefall::CodefallHandler;
use super::command::Command;

#[derive(Debug)]
pub struct CommandHandler {
    triggers: Regex,
    client: IrcClient,

    codefall: CodefallHandler,
    announce: Vec<String>,
}

impl CommandHandler {
    pub async fn new() -> Result<Self> {
        let triggers = Regex::new(&env::var("PUMP19_TRIGGERS")?)?;

        info!("Setting up IRC client…");
        let config = IrcConfig {
            nickname: Some(env::var("PUMP19_IRC_NICKNAME")?),
            username: Some(env::var("PUMP19_IRC_USERNAME")?),
            password: Some(env::var("PUMP19_IRC_PASSWORD")?),
            channels: env::var("PUMP19_IRC_CHANNELS")?
                .split(',')
                .map(|c| c.to_owned())
                .collect(),

            realname: Some("Rusty Golem".to_owned()),
            server: Some("irc.chat.twitch.tv".to_owned()),
            port: Some(6697),

            ..IrcConfig::default()
        };
        let client = IrcClient::from_config(config).await?;
        client.identify()?;

        let codefall = CodefallHandler::new().await?;
        let announce = env::var("PUMP19_CODEFALL_ANNOUNCE")?
            .split(',')
            .map(|c| c.to_owned())
            .collect();

        let cmd_hdl = CommandHandler {
            triggers,
            client,
            codefall,
            announce,
        };

        Ok(cmd_hdl)
    }

    pub async fn run(&mut self) -> Result<()> {
        #[derive(Debug)]
        enum Message {
            IRC(IrcMessage),
            Codefall(String),
        }

        let mut stream = stream::select(
            self.codefall
                .key_stream()
                .await?
                .map(|c| Message::Codefall(c)),
            self.client.stream()?.map(|m| Message::IRC(m.unwrap())),
        );

        while let Some(message) = stream.next().await {
            match message {
                Message::Codefall(key) => {
                    debug!("Received codefall notification for key: {}", key);
                    self.announce_codefall(&key).await?;
                }

                Message::IRC(message) => {
                    debug!("Received IRC message: {}", message);
                    if let (
                        Some(Prefix::Nickname(nickname, _, _)),
                        IrcCommand::PRIVMSG(channel, message),
                    ) = (message.prefix, message.command)
                    {
                        self.process_privmsg(&nickname, &channel, &message).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_privmsg(&self, nickname: &str, channel: &str, message: &str) -> Result<()> {
        let command = match self.triggers.find(message) {
            None => return Ok(()),
            Some(trigger_match) => message.get(trigger_match.end()..).unwrap(),
        };

        info!("Got command from {} on {}: {}", nickname, channel, command);
        if let Some(response) = match Command::from(command) {
            Command::Help => Some(self.handle_help()),
            Command::Bingo => Some(self.handle_bingo()),
            Command::Codefall(limit) => self.handle_codefall(nickname, limit).await,

            Command::Unknown => None,
        } {
            self.client.send_privmsg(channel, response)?;
        }

        Ok(())
    }

    fn handle_help(&self) -> String {
        format!(
            "Pump19 is run by Twisted Pear. Check {} for a list of supported commands.",
            env::var("PUMP19_URL_HELP").unwrap()
        )
    }

    fn handle_bingo(&self) -> String {
        format!(
            "Check out {} for our interactive Trope Bingo cards.",
            env::var("PUMP19_URL_BINGO").unwrap()
        )
    }

    async fn handle_codefall(&self, user: &str, limit: u32) -> Option<String> {
        let codes = self.codefall.random_entries(user, limit).await;
        match codes {
            Err(err) => {
                error!("Could not query unclaimed codes: {}", err);
                None
            }
            Ok(codes) => {
                if codes.is_empty() {
                    return Some(format!(
                        "Could not find any unclaimed codes. Visit {} to add new entries",
                        env::var("PUMP19_URL_CODEFALL").unwrap()
                    ));
                }

                Some(format!(
                    "Codefall | {}",
                    codes
                        .iter()
                        .map(|c| format!("{}", c))
                        .collect::<Vec<_>>()
                        .join(" | ")
                ))
            }
        }
    }

    async fn announce_codefall(&self, key: &str) -> Result<()> {
        let code = self.codefall.entry(key).await?;

        for channel in &self.announce {
            self.client
                .send_privmsg(channel, format!("Codefall | {}", code))?;
        }

        Ok(())
    }
}
