use ::slug::slugify;
use anyhow::anyhow;
use handlebars::{
    handlebars_helper, Context as HandlebarsContext, Handlebars, Helper, Output, RenderContext,
    RenderError, RenderErrorReason,
};
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
// one name or a list, displayed as a sentence: "Mary Ann Smith and Jane Farmer"
handlebars_helper!(people: |names: Json|
    web::people::display_names(names)
);

// {{date_format date}} or {{date_format date "%b %-d, %Y"}}
// a value that isn't a valid date is written as is
fn date_format_helper(
    h: &Helper<'_>,
    _: &Handlebars,
    _: &HandlebarsContext,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let Some(value) = h.param(0).map(|p| p.value()) else {
        return Ok(());
    };
    let format = match h.param(1) {
        None => None,
        Some(p) => Some(
            p.value()
                .as_str()
                .ok_or(RenderErrorReason::InvalidParamType("date format string"))?,
        ),
    };
    let text = match web::date::FmDate::from_value(value) {
        Some(date) => date
            .format(format)
            .map_err(|e| RenderErrorReason::Other(e.to_string()))?,
        None => match value {
            serde_json::Value::Null => String::new(),
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        },
    };
    out.write(&handlebars::html_escape(&text))?;
    Ok(())
}

fn helper_context_get_string_from_key(hc: &HandlebarsContext, key: &str) -> String {
    if let Some(map) = hc.data().as_object() {
        if let Some(value) = map.get(key) {
            if let Some(s) = value.as_str() {
                return s.to_string();
            }
        }
    }
    String::from("")
}
fn person_helper(
    h: &Helper<'_>,
    _: &Handlebars,
    hc: &HandlebarsContext,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let param0 = h
        .param(0)
        .ok_or(RenderErrorReason::ParamNotFoundForIndex("name", 0))?;
    let name = param0
        .value()
        .as_str()
        .ok_or(RenderErrorReason::InvalidParamType("name string"))?;
    // "Vasquez, Maria" displays and links as "Maria Vasquez"
    let name = web::people::display_name(name);
    let baseurl = helper_context_get_string_from_key(hc, "baseurl");
    let output = format!(
        "<a href='{}/people/{}'>{}</a>",
        baseurl,
        slugify(&name),
        name
    );
    out.write(&output)?;

    Ok(())
}

pub fn init<'a>(config: &'a Config) -> anyhow::Result<Context<'a>> {
    info!("init_templates");
    clean_and_recreate_dir(&config.builddir)?;
    let buildtemplatedir = config.buildtemplatedir();
    copy_dir_all(&config.templatedir, &buildtemplatedir)?;
    let buildrefdir = buildtemplatedir.join("ref");
    std::fs::create_dir_all(&buildrefdir).map_err(|e| {
        anyhow!(format!(
            "failed to create directory: {}, error: {}",
            &buildrefdir.display(),
            e
        ))
    })?;

    let ref_dir = config
        .sourcedir
        .canonicalize()?
        .parent()
        .unwrap()
        .join("ref");
    web::Ref::process_markdown(
        config,
        ref_dir,
        &buildtemplatedir.canonicalize()?.join("ref"),
    )?;

    let buildtemplatedir = config.buildtemplatedir();
    info!("buildtemplatedir: {}", buildtemplatedir.display());
    let mut hbs = Handlebars::new();
    hbs.register_helper("date_format", Box::new(date_format_helper));
    hbs.register_helper("people", Box::new(people));
    hbs.register_helper("person", Box::new(person_helper));
    hbs.register_helper("slug", Box::new(slug));
    hbs.register_helper("split", Box::new(split));
    hbs.register_templates_directory(&buildtemplatedir, Default::default())
        .map_err(|e| {
            anyhow!(
                "failed to register template directory, error {:?}. directory: {}",
                e,
                buildtemplatedir.display()
            )
        })?;
    info!(
        "Setup: template directory '{}' registered",
        &buildtemplatedir.display()
    );

    Ok(Context { config, hbs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_person_helper() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("person", Box::new(person_helper));
        let mut data: HashMap<String, String> = HashMap::new();
        data.insert(String::from("baseurl"), String::from("https://example.com"));
        assert_eq!(
            hbs.render_template("{{ person \"Ada Lovelace\" }}", &data)
                .unwrap(),
            "<a href='https://example.com/people/ada-lovelace'>Ada Lovelace</a>"
        );
    }

    #[test]
    fn test_person_helper_last_first() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("person", Box::new(person_helper));
        let data: HashMap<String, String> = HashMap::new();
        assert_eq!(
            hbs.render_template("{{ person \"Vasquez, Maria\" }}", &data)
                .unwrap(),
            "<a href='/people/maria-vasquez'>Maria Vasquez</a>"
        );
    }

    #[test]
    fn test_people_helper() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("people", Box::new(people));
        let data = serde_json::json!({
            "one": "Smith, Mary Ann",
            "two": ["Smith, Mary Ann", "Farmer, Jane"],
        });
        assert_eq!(
            hbs.render_template("{{ people one }}", &data).unwrap(),
            "Mary Ann Smith"
        );
        assert_eq!(
            hbs.render_template("{{ people two }}", &data).unwrap(),
            "Mary Ann Smith and Jane Farmer"
        );
        assert_eq!(
            hbs.render_template("{{ people missing }}", &data).unwrap(),
            ""
        );
    }

    #[test]
    fn test_date_format_helper() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("date_format", Box::new(date_format_helper));
        let data = serde_json::json!({
            "year": 1984,
            "day": "2025-11-08",
            "text": "Nov. 8, 2025",
        });
        let render = |t: &str| hbs.render_template(t, &data).unwrap();
        assert_eq!(render("{{ date_format year }}"), "1984");
        assert_eq!(render("{{ date_format day }}"), "November 8, 2025");
        assert_eq!(render("{{ date_format day \"%b %-d\" }}"), "Nov 8");
        assert_eq!(render("{{ date_format text }}"), "Nov. 8, 2025");
        assert_eq!(render("{{ date_format missing }}"), "");
        assert!(hbs
            .render_template("{{ date_format day \"%Q\" }}", &data)
            .is_err());
    }

    #[test]
    fn test_split_helper() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("split", Box::new(split));
        let mut data: HashMap<String, String> = HashMap::new();
        let title = String::from(
            "First Line
Second Line",
        );
        data.insert(String::from("title"), title);
        let template = "{{#each (split title)}}
<h1>{{ this }}</h1>
{{/each}}";
        assert_eq!(
            hbs.render_template(template, &data).unwrap(),
            "<h1>First Line</h1>\n<h1>Second Line</h1>\n"
        );
    }

    #[test]
    fn test_slug_helper() {
        let mut hbs = Handlebars::new();
        hbs.register_helper("slug", Box::new(slug));
        let data: HashMap<String, String> = HashMap::new();
        assert_eq!(
            hbs.render_template("{{ slug \"Ada Lovelace\" }}", &data)
                .unwrap(),
            "ada-lovelace"
        );
    }
}
