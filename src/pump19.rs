use std::fs;
use std::path::Path;

use anyhow::Result;
use futures::prelude::*;
use irc::client::data::Config as IrcConfig;
use irc::client::prelude::*;
use log::{debug, error, info};
use regex::Regex;
use serde::Deserialize;

const HELP_URL: &str = "https://pump19.eu/commands";
const BINGO_URL: &str = "https://pump19.eu/bingo";
const CODEFALL_URL: &str = "https://pump19.eu/codefall";

#[derive(Deserialize)]
pub struct Config {
    triggers: String,
    database: String,

    irc: IrcConfig,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let data = fs::read_to_string(path)?;
        let cfg: Config = toml::from_str(&data)?;

        Ok(cfg)
    }
}

pub struct Golem {
    config: Config,
    cmdhdl: CommandHandler,
}

impl Golem {
    pub async fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config = Config::load(config_path)?;

        Golem::from_config(config).await
    }

    pub async fn from_config(config: Config) -> Result<Self> {
        let cmdhdl = CommandHandler::new(&config.triggers, &config.database).await?;

        Ok(Golem { config, cmdhdl })
    }

    pub async fn run(self: Self) -> Result<()> {
        let mut client = Client::from_config(self.config.irc).await?;
        client.identify()?;

        let mut stream = client.stream()?;
        while let Some(message) = stream.next().await.transpose()? {
            debug!("IRC message: {}", message);

            if let Some(Prefix::Nickname(nickname, _, _)) = message.prefix {
                if let Command::PRIVMSG(channel, message) = message.command {
                    self.cmdhdl
                        .process_privmsg(&client, &nickname, &channel, &message)
                        .await?;
                }
            }
        }

        Ok(())
    }
}

lazy_static! {
    static ref RE_CODEFALL: Regex = Regex::new(r"codefall(?: (?P<limit>[1-3]))?").unwrap();
}
enum GolemCommand {
    Help,
    Bingo,
    Codefall(u32),

    Unknown,
}

impl From<&str> for GolemCommand {
    fn from(command: &str) -> Self {
        match command {
            "help" => GolemCommand::Help,
            "bingo" => GolemCommand::Bingo,
            command if RE_CODEFALL.is_match(command) => {
                let caps = RE_CODEFALL.captures(command).unwrap();
                let limit = caps
                    .name("limit")
                    .map_or(1, |l| l.as_str().parse().unwrap_or(1));

                GolemCommand::Codefall(limit)
            }

            _ => GolemCommand::Unknown,
        }
    }
}
struct CommandHandler {
    triggers: Regex,
    database: tokio_postgres::Client,
}

impl CommandHandler {
    async fn new(triggers: &str, database: &str) -> Result<Self> {
        let triggers = Regex::new(triggers)?;

        debug!("Setting up database connection…");
        let (database, connection) =
            tokio_postgres::connect(database, tokio_postgres::NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Connection error: {}", e);
            }
        });

        let command_handler = CommandHandler { triggers, database };

        Ok(command_handler)
    }

    async fn process_privmsg(
        self: &Self,
        client: &Client,
        nickname: &str,
        channel: &str,
        message: &str,
    ) -> Result<()> {
        let command = match self.triggers.find(message) {
            None => return Ok(()),
            Some(trigger_match) => message.get(trigger_match.end()..).unwrap(),
        };

        info!("Got command from {} on {}: {}", nickname, channel, command);
        if let Some(response) = match GolemCommand::from(command) {
            GolemCommand::Help => Some(self.handle_help()),
            GolemCommand::Bingo => Some(self.handle_bingo()),
            GolemCommand::Codefall(limit) => self.handle_codefall(nickname, limit).await,

            GolemCommand::Unknown => None,
        } {
            client.send_privmsg(channel, response).unwrap();
        }

        Ok(())
    }

    fn handle_help(self: &Self) -> String {
        format!(
            "Pump19 is run by Twisted Pear. Check {} for a list of supported commands.",
            HELP_URL
        )
    }

    fn handle_bingo(self: &Self) -> String {
        format!(
            "Check out {} for our interactive Trope Bingo cards.",
            BINGO_URL
        )
    }

    async fn handle_codefall(self: &Self, user: &str, limit: u32) -> Option<String> {
        let rows = self.database
            .query("SELECT description, code_type, key FROM codefall_unclaimed WHERE user_name = $1::TEXT ORDER BY random() LIMIT $2::OID", &[&user, &limit])
            .await;

        let codes = match rows {
            Err(err) => {
                error!("Could not query unclaimed codes: {}", err);
                return None;
            }
            Ok(codes) => codes,
        };

        if codes.is_empty() {
            return Some(format!(
                "Could not find any unclaimed codes. Visit {} to add new entries",
                CODEFALL_URL
            ));
        }

        Some(format!(
            "Codefall | {}",
            codes
                .iter()
                .map(|r| format!(
                    "{} ({}) {}/{}",
                    r.get::<&str, &str>("description"),
                    r.get::<&str, &str>("code_type"),
                    CODEFALL_URL,
                    r.get::<&str, &str>("key")
                ))
                .collect::<Vec<String>>()
                .join(" | ")
        ))
    }
}
