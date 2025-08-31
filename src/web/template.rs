use anyhow::anyhow;
use handlebars::{Handlebars,handlebars_helper};
use ::slug::slugify;
use tracing::info;

use crate::{
    config::{Config, Context},
    util::*,
    web,
};

handlebars_helper!(slug: |input:String|
    slugify(input)
);
handlebars_helper!(split: |input:String, {sep:str="\n"}|
        input.split(sep).collect::<Vec<&str>>()
);

pub fn init<'a>(config: &'a Config) -> anyhow::Result<Context<'a>> {
    info!("init_templates");
    clean_and_recreate_dir(&config.builddir)?;
    let buildtemplatedir = config.buildtemplatedir();
    copy_dir_all(&config.templatedir, &buildtemplatedir)?;
    let buildrefdir = buildtemplatedir.join("ref");
    std::fs::create_dir_all(&buildrefdir).map_err(|e| {
        anyhow!(format!("failed to create directory: {}, error: {}", &buildrefdir.display(), e))
    })?;

    let ref_dir = config.sourcedir.canonicalize()?.parent().unwrap().join("ref");
    web::Ref::process_markdown(config, ref_dir, &buildtemplatedir.canonicalize()?.join("ref"))?;

    let buildtemplatedir = config.buildtemplatedir();
    info!("buildtemplatedir: {}", buildtemplatedir.display());
    let mut hbs = Handlebars::new();
    hbs.register_helper("slug", Box::new(slug));
    hbs.register_helper("split", Box::new(split));
    hbs.register_templates_directory(&buildtemplatedir, Default::default())
        .map_err(|e| {
            anyhow!("failed to register template directory, error {:?}. directory: {}", e, buildtemplatedir.display())
        })?;
    info!("Setup: template directory '{}' registered", &buildtemplatedir.display());

    Ok(Context {
        config, hbs
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_split_helper() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("split", Box::new(split));
        let mut data: HashMap<String, String> = HashMap::new();
        let title = String::from("First Line
Second Line");
        data.insert(String::from("title"), title);
        let template =
"{{#each (split title)}}
<h1>{{ this }}</h1>
{{/each}}";
        assert_eq!(
            hbs.render_template(&template, &data).unwrap(),
            "<h1>First Line</h1>\n<h1>Second Line</h1>\n"
        );
    }

        #[test]
    fn test_slug_helper() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("slug", Box::new(slug));
        let data: HashMap<String, String> = HashMap::new();
        assert_eq!(
            hbs.render_template("{{ slug \"Ada Lovelace\" }}", &data).unwrap(),
            "ada-lovelace"
        );
    }
}

