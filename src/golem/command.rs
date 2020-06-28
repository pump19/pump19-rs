// Copyright (c) 2020 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_CODEFALL: Regex = Regex::new(r"codefall(?: (?P<limit>[1-3]))?").unwrap();
}

pub enum Command {
    Help,
    Bingo,
    Codefall(u32),

    Unknown,
}

impl From<&str> for Command {
    fn from(command: &str) -> Self {
        match command {
            "help" => Command::Help,
            "bingo" => Command::Bingo,
            command if RE_CODEFALL.is_match(command) => {
                let caps = RE_CODEFALL.captures(command).unwrap();
                let limit = caps
                    .name("limit")
                    .map_or(1, |l| l.as_str().parse().unwrap_or(1));

                Command::Codefall(limit)
            }

            _ => Command::Unknown,
        }
    }
}

mod test {}
