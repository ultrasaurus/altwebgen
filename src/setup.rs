use anyhow::anyhow;
use handlebars::Handlebars;
use tracing::info;

use crate::{
    config::Config,
    util::*,
    web,
};


pub fn init_templates(config: &Config, hbs: &mut Handlebars) -> anyhow::Result<()> {
    info!("init_templates");
    clean_and_recreate_dir(&config.builddir)?;
    let buildtemplatedir = config.buildtemplatedir();
    copy_dir_all(&config.templatedir, &buildtemplatedir)?;
    let buildrefdir = buildtemplatedir.join("ref");
    std::fs::create_dir_all(&buildrefdir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &buildrefdir.display(), e))
    })?;

    web::Ref::process_markdown("ref", &buildtemplatedir.join("ref"))?;

    let buildtemplatedir = config.buildtemplatedir();
    hbs.register_templates_directory(&buildtemplatedir, Default::default())
        .map_err(|_| {
            anyhow!("failed to register template directory: {}", buildtemplatedir.display())
        })?;
    info!("Setup: template directory '{}' registered", &buildtemplatedir.display());

    Ok(())
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

pub fn clean_build(config: &Config, hbs: &mut Handlebars) -> anyhow::Result<()> {
    create_source_dirs(&config)?;
    clean_and_recreate_dir(&config.outdir)?;

    if let Err(e) = init_templates(config, hbs) {
        return Err(e.context("setting up templates failed"))
    };

    if let Err(e) = web::process_files(&config, &hbs) {
        return Err(e.context("build failure"))
    };

    Ok(())
}

// initial setup, called only once
pub fn init(config: &Config) -> anyhow::Result<Handlebars<'static>> {
    info!("init: start");
    info!("      working directory {}", get_current_working_dir()?.display());
    let mut hbs = Handlebars::new();
    create_source_dirs(&config)?;

    clean_build(&config, &mut hbs)?;
    info!("init: complete");
    Ok(hbs)
}
