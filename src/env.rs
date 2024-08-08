use std::env;

pub struct EnvOptions {
    pub port: usize,
    pub model_path: String,
    pub nstates: usize,
}

pub fn get_opts() -> EnvOptions {
    let port = match env::var("PORT") {
        Ok(value) => {
            let res = match value.parse::<usize>() {
                Ok(value_usize) => value_usize,
                Err(_) => {
                    panic!("Port is invalid");
                }
            };

            res
        }
        Err(_) => 5000,
    };

    let model_path = match env::var("MODEL_PATH") {
        Ok(value) => value,
        Err(_) => "./models/ggml-large.bin".to_string(),
    };

    let nstates = match env::var("NSTATES") {
        Ok(value) => {
            let res = match value.parse::<usize>() {
                Ok(state_usize) => state_usize,
                Err(_) => {
                    panic!("Invalid nstates")
                }
            };

            res
        }
        Err(_) => 1,
    };

    EnvOptions {
        port,
        model_path,
        nstates,
    }
}
