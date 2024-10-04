use std::collections::HashMap;
use std::path::{Path,PathBuf};
use tracing::{error, info};

#[derive(Clone)]
pub struct Config {
    pub outdir: PathBuf,
    pub builddir: PathBuf,
    pub sourcedir: PathBuf,
    pub templatedir: PathBuf,
    pub site_attr: HashMap<String, String>
}

impl Config {
    // given sourcepath: some path inside sourcedir
    // return: new parallel path inside outdir
    pub fn outpath<P: AsRef<Path>>(&self, sourcepath: P) -> std::io::Result<PathBuf> {
        let rel_path = sourcepath.as_ref()
            .strip_prefix(&self.sourcedir)
            .expect("strip prefix match");
        Ok(self.outdir.join(rel_path))
    }
    pub fn buildtemplatedir(&self) -> PathBuf {
         self.builddir.join("template")
    }

    pub fn new(outdir_str: &str,
           sourcedir_str: &str,
           templatedir_str: &str,
    ) -> Config {
        info!("config...");
        info!("   outdir:      {}", outdir_str);
        info!("   sourcedir:   {}", sourcedir_str);
        info!("   templatedir: {}", templatedir_str);

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

        // build dir and sub-directories not configurable
        info!("   -- additional directories not user configurable -- ");
        let build_str = ".build";
        let builddir = std::path::PathBuf::from(build_str);
        info!("   builddir: {}", builddir.display());
        if !builddir.exists() {
            let _result = std::fs::create_dir_all(&builddir);
            if _result.is_err() {
                error!("could not create default build director '.build'");
            }
        }

        // create template directory if not present
        let buildtemplatedir = builddir.join("template");
        info!("   buildtemplatedir: {}", buildtemplatedir.display());
        if !buildtemplatedir.exists() {
            if let Err(e) = std::fs::create_dir_all(&builddir) {
                error!("could not create build template directory: {}, error: {}", buildtemplatedir.display(), e)
            };
        }

        Config {
            outdir,
            builddir,
            sourcedir,
            templatedir: PathBuf::from("template"),
            site_attr,
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
        Config::new(".dist", "source", "template")
    }
}
