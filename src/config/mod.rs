use std::collections::HashMap;
use std::path::{Path,PathBuf};
use tracing::{error, info};

// export types for config fields
mod mode;
pub use mode::Mode as Mode;
mod transcript;
pub use transcript::Transcript as Transcript;

#[derive(Clone, Debug)]
pub struct Context<'a> {
    pub config: &'a Config,
    pub hbs: handlebars::Handlebars<'a>
}

#[derive(Clone, Debug)]
pub struct Config {
    pub outdir: PathBuf,
    pub builddir: PathBuf,
    pub sourcedir: PathBuf,
    pub templatedir: PathBuf,
    pub site_attr: HashMap<String, String>,
    pub prefix: String,
    pub mode: Mode,
    pub transcript: Transcript
}

// ensure prefix starts and ends with '/'
fn root_prefix_format(path_prefix: &str) -> String {
    if path_prefix == "" {
        String::from("/")
    } else {
        let start = if path_prefix.starts_with('/') { "" } else { "/" };
        let end = if path_prefix.ends_with('/') { "" } else { "/" };
        format!("{}{}{}", start, path_prefix, end)
    }
}


impl Config {
    // given sourcepath: some path inside sourcedir
    // return: new parallel path inside outdir
    pub fn outpath<P: AsRef<Path>>(&self, sourcepath: P) -> std::io::Result<PathBuf> {
        let rel_path = sourcepath.as_ref()
            .strip_prefix(&self.sourcedir)
            .expect("sourcedir strip prefix match");
        Ok(self.outdir.join(rel_path))
    }

    // used for when templates have associated media files or css
    // so they can be kept in the template directory
    // when building, they are automatically copied to the outdir
    pub fn templatedir_outpath<P: AsRef<Path>>(&self, sourcepath: P) -> std::io::Result<PathBuf> {
        let rel_path = sourcepath.as_ref()
            .strip_prefix(&self.templatedir)
            .expect("templatedir strip prefix match");
        Ok(self.outdir.join(rel_path))
    }

    pub fn buildtemplatedir(&self) -> PathBuf {
         self.builddir.join("template")
    }

    pub fn new(outdir_str: &str,
           sourcedir_str: &str,
           templatedir_str: &str,
           path_prefix: &str,
           mode: Mode,
           transcript: Transcript
    ) -> Config {
        info!("config...");
        info!("   outdir:      {}", outdir_str);
        info!("   sourcedir:   {}", sourcedir_str);
        info!("   templatedir: {}", templatedir_str);
        info!("   prefix: {}",      path_prefix);
        info!("   mode: {}",      mode);
        info!("   transcript: {}",   transcript);

        // create output directory if not present
        let outdir: PathBuf = std::path::PathBuf::from(outdir_str);
        if !outdir.exists() {
            let _result = std::fs::create_dir_all(&outdir);
            if _result.is_err() {
                error!("could not create default output director '.dist'");
            }
        }

        // create source directory if not present
        let sourcedir = PathBuf::from(sourcedir_str);
        let site_attr = match read_site_yaml(&sourcedir) {
            Ok(h) => h,
            Err(e) => {
                error!("could not read _site.yaml from {}, {:?}", &sourcedir.display(), e);
                HashMap::new()
            }
        };

        // create template directory if not present
        let templatedir: PathBuf = std::path::PathBuf::from(templatedir_str);
        if !templatedir.exists() {
            let _result = std::fs::create_dir_all(&templatedir);
            if _result.is_err() {
                error!("could not create templatedir directory");
            }
        }


        // build dir and sub-directories not configurable
        info!("   -- additional directories not user configurable -- ");
        info!("   -- builddir will be created as sibling of sourcedir -- ");
        let build_str = ".build";
        let builddir = sourcedir.parent().unwrap().join(build_str);
        info!("   builddir: {}", builddir.display());
        if !builddir.exists() {
            let _result = std::fs::create_dir_all(&builddir);
            if _result.is_err() {
                error!("could not create default build director '.build'");
            }
        }

        // create build template directory if not present
        let buildtemplatedir = builddir.join("template");
        info!("   buildtemplatedir: {}", buildtemplatedir.display());
        if !buildtemplatedir.exists() {
            if let Err(e) = std::fs::create_dir_all(&builddir) {
                error!("could not create build template directory: {}, error: {}", buildtemplatedir.display(), e)
            };
        }

        // ensure prefix starts and ends with '/'
        let prefix = root_prefix_format(path_prefix);

        Config {
            outdir,
            builddir,
            sourcedir,
            templatedir,
            site_attr,
            prefix: prefix.to_string(),
            mode,
            transcript
        }

    }

}

fn read_site_yaml(sourcedir: &Path) -> anyhow::Result<HashMap<String, String>> {

    let site_yaml_path = sourcedir.join("_site.yaml");
    let site_attr:HashMap<String, String>  = if !site_yaml_path.exists() {
        HashMap::new()
    } else {
        let f = std::fs::File::open(&site_yaml_path)?;
        serde_yaml::from_reader(f)?
    };
    Ok(site_attr)
}

impl Default for Config {

     fn default() -> Config {
        info!("default config");
        Config::new(".dist",
            "source",
            "template",
            "",
            Mode::Build, Transcript::Static)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_format_empty() {
        let result = root_prefix_format("");
        assert_eq!(result, String::from("/"));
    }

    #[test]
    fn test_prefix_format_correct() {
        let given_prefix = "/whatever/";
        let result = root_prefix_format(given_prefix);
        assert_eq!(result, String::from(given_prefix));
    }

    #[test]
    fn test_prefix_format_correct_internal_slash() {
        let given_prefix = "/what/ever/";
        let result = root_prefix_format(given_prefix);
        assert_eq!(result, String::from(given_prefix));
    }


    #[test]
    fn test_prefix_format_missing_end_slash() {
        let result = root_prefix_format("/thing");
        assert_eq!(result, String::from("/thing/"));
    }

    #[test]
    fn test_prefix_format_missing_start_slash() {
        let result = root_prefix_format("start/");
        assert_eq!(result, String::from("/start/"));
    }

}