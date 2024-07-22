use std::fs;
use std::path::Path;

use crate::env::EnvOptions;

pub fn check_and_fix(opts: &EnvOptions) {
    let tmpfile_dir_path = Path::new("./tmpfiles");
    let model_path = Path::new(&opts.model_path);

    if !tmpfile_dir_path.exists() {
        fs::create_dir("./tmpfiles").unwrap();
    }

    if !model_path.exists() {
        panic!("model path {} does not exist", opts.model_path);
    }
}
