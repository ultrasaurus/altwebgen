use anyhow::anyhow;
use handlebars::Handlebars;
use tracing::info;

use crate::{
    config::{Config, Context},
    util::*,
    web,
};


pub fn init_templates<'a>(config: &'a Config) -> anyhow::Result<Context<'a>> {
    info!("init_templates");
    clean_and_recreate_dir(&config.builddir)?;
    let buildtemplatedir = config.buildtemplatedir();
    copy_dir_all(&config.templatedir, &buildtemplatedir)?;
    let buildrefdir = buildtemplatedir.join("ref");
    std::fs::create_dir_all(&buildrefdir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &buildrefdir.display(), e))
    })?;

    web::Ref::process_markdown(config, "ref", &buildtemplatedir.join("ref"))?;

    let buildtemplatedir = config.buildtemplatedir();
    info!("buildtemplatedir: {}", buildtemplatedir.display());
    let mut hbs = Handlebars::new();
    hbs.register_templates_directory(&buildtemplatedir, Default::default())
        .map_err(|_| {
            anyhow!("failed to register template directory: {}", buildtemplatedir.display())
        })?;
    info!("Setup: template directory '{}' registered", &buildtemplatedir.display());

    Ok(Context {
        config, hbs
    })
}

fn create_source_dirs(config: &Config) -> anyhow::Result<()> {
    std::fs::create_dir_all(&config.sourcedir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &config.sourcedir.display(), e))
    })?;
    let refdir = "ref";    // TODO: config?
    std::fs::create_dir_all(refdir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", refdir, e))
    })?;
    std::fs::create_dir_all(&config.templatedir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &config.templatedir.display(), e))
    })?;

    Ok(())
}

pub fn clean_build(config: &Config) -> anyhow::Result<Context> {
    create_source_dirs(&config)?;
    clean_and_recreate_dir(&config.outdir)?;

    match init_templates(config) {
        Err(e) => return Err(e.context("setting up templates failed")),
        Ok(context) => {
            if let Err(e) = web::process_files(&context) {
                return Err(e.context("build failure"))
            }
            Ok(context)
        }
    }
}

// initial setup, called only once
pub fn init_and_build(config: &Config) -> anyhow::Result<Context> {
    info!("init: start");
    info!("      working directory {}", get_current_working_dir()?.display());
    create_source_dirs(&config)?;

    let context = clean_build(&config)?;
    info!("init: complete");
    Ok(context)
}
