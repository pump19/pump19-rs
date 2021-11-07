// Copyright (c) 2021 Kevin Perry <perry at pump19 dot eu>
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::env;
use std::fmt;

use anyhow::Result;
use futures::{Stream, StreamExt};
use log::info;
use sqlx::{
    postgres::{PgListener, PgPool, PgPoolOptions},
    query_as,
};

#[derive(Debug)]
pub struct Code {
    description: String,
    code_type: String,
    key: String,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) {}/{}",
            self.description,
            self.code_type,
            env::var("PUMP19_URL_CODEFALL").unwrap(),
            self.key
        )
    }
}

#[derive(Debug)]
pub struct CodefallHandler {
    database: PgPool,
}

impl CodefallHandler {
    pub async fn new() -> Result<Self> {
        info!("Setting up database connection…");
        let database = PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var("PUMP19_CODEFALL_DATABASE")?)
            .await?;

        Ok(CodefallHandler { database })
    }

    pub async fn key_stream(&self) -> Result<impl Stream<Item = Result<String>>> {
        let mut listener = PgListener::connect_with(&self.database).await?;
        listener.listen("codefall").await?;

        Ok(listener.into_stream().map(|n| Ok(n?.payload().to_owned())))
    }

    pub async fn random_entries(&self, user: &str, limit: u32) -> Result<Vec<Code>> {
        let codes = query_as!(
            Code,
            r"
SELECT description, code_type, key
FROM codefall_unclaimed
WHERE user_name = $1::TEXT
ORDER BY random()
LIMIT $2::OID",
            user,
            limit
        )
        .fetch_all(&self.database)
        .await?;

        Ok(codes)
    }

    pub async fn entry(&self, key: &str) -> Result<Code> {
        let code = query_as!(
            Code,
            r"
SELECT description, code_type, key
FROM codefall_unclaimed
WHERE key = $1::TEXT
LIMIT 1",
            key
        )
        .fetch_one(&self.database)
        .await?;

        Ok(code)
    }
}
