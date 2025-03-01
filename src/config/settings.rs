use lazy_static::lazy_static;
use std::env;

pub struct Config {
    pub discord_token: String,
    pub download_dir: String,
    pub converted_dir: String,
    pub host: String,
    pub port: u16,
    pub public_url: String,
}

impl Config {
    pub fn new() -> Self {
        let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");

        let data_path = env::var("DATA_PATH").unwrap_or(".".to_string());
        let download_dir = format!("{}/downloads", data_path);
        let converted_dir = format!("{}/converted", data_path);

        let host = env::var("WEBSERVER_HOST").unwrap_or("127.0.0.1".to_string());
        let port = env::var("WEBSERVER_PORT")
            .unwrap_or("8080".to_string())
            .parse()
            .expect("WEBSERVER_PORT must be a valid u16");

        let public_url = env::var("PUBLIC_URL").unwrap_or(format!("http://{host}:{port}"));

        Self {
            discord_token,
            public_url,
            download_dir,
            converted_dir,
            host,
            port,
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
