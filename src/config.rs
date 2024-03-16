use std::path::{Path,PathBuf};
use tracing::error;

#[derive(Clone)]
pub struct Config {
    pub outdir: PathBuf,
    pub sourcedir: PathBuf,
}

impl Config {
    pub fn new<P: AsRef<Path>>(outdir: P, sourcedir:P) -> Config {
        Config {
            outdir: outdir.as_ref().to_path_buf(),
            sourcedir: sourcedir.as_ref().to_path_buf()
        }
    }

    // given sourcepath: some path inside sourcedir
    // return: new parallel path inside outdir
    pub fn outpath<P: AsRef<Path>>(&self, sourcepath: P) -> std::io::Result<PathBuf> {
        let rel_path = sourcepath.as_ref()
            .strip_prefix(&self.sourcedir)
            .expect("strip prefix match");
        Ok(self.outdir.join(rel_path))
    }

}

impl Default for Config {
     fn default() -> Config {
        let outdir = std::path::PathBuf::from(".dist");
        if !outdir.exists() {
            let _result = std::fs::create_dir_all(outdir);
            if _result.is_err() {
                error!("could not create default output director '.dist'");
            }
        }

        Config::new(".dist", "source")
    }
}
