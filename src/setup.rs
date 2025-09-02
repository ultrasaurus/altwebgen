use tracing::info;

use crate::{
    config::{Config, Context},
    util::*,
    web,
};

pub fn clean_build(config: &Config) -> anyhow::Result<Context> {
    config.create_source_dirs()?;
    clean_and_recreate_dir(&config.outdir)?;

    match web::template::init(config) {
        Err(e) => return Err(e.context("setting up templates failed")),
        Ok(context) => {
            if let Err(e) = web::process_files(&context) {
                return Err(e.context("build failure"))
            } else {
                   info!("...build compelte!");
            }
            Ok(context)
        }
    }
}

// initial setup, called only once
pub fn init_and_build(config: &Config) -> anyhow::Result<Context> {
    info!("init: start");
    info!("      working directory {}", get_current_working_dir()?.display());
    config.create_source_dirs()?;

    let context = clean_build(&config)?;
    info!("init: complete");
    Ok(context)
}
