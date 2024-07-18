use std::env;

pub struct EnvOptions {
    pub port: usize,
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

    EnvOptions { port }
}
