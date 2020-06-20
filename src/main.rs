mod pump19;

use anyhow::Result;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    log::info!("Starting rust golem…");

    let xdg_dirs = xdg::BaseDirectories::with_prefix("pump19.rs")
        .expect("Cannot determine XDG base directories");
    let cfg_path = xdg_dirs
        .find_config_file("config.toml")
        .expect("Cannot find config file");

    log::info!("Found configuration file {:?}", cfg_path);

    let golem = pump19::Golem::new(cfg_path)
        .await
        .expect("Cannot create golem");

    log::info!("Running pump19 rust golem…");

    golem.run().await
}
