// Copyright (c) 2020 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_CODEFALL: Regex = Regex::new(r"codefall(?: (?P<limit>[1-3]))?").unwrap();
    static ref RE_MULTIPLE: Regex =
        Regex::new(r"mult(?:i(?:pl(?:y|es?))?)? (?:\$)?(?P<value>[0-9]+(?:\.[0-9]{1,2})?)")
            .unwrap();
}

pub enum Command {
    Help,
    Bingo,

    Codefall(u32),
    Multiple(f64),

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

            command if RE_MULTIPLE.is_match(command) => {
                let caps = RE_MULTIPLE.captures(command).unwrap();
                let value = caps
                    .name("value")
                    .unwrap()
                    .as_str()
                    .parse()
                    .expect("Cannot parse as f64");

                Command::Multiple(value)
            }

            _ => Command::Unknown,
        }
    }
}

mod test {}
