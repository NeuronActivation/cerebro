use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;

pub struct Config {
    pub discord_token: String,
    pub server_url: String,
    pub data_path: String,
    pub download_dir: String,
    pub converted_dir: String,
    pub webserver_host: String,
    pub webserver_port: u16,
}

impl Config {
    pub fn new() -> Self {
        let discord_token =
            env::var("DISCORD_TOKEN").expect("Discord token not set in the environment");

        let server_url = env::var("WEBSERVER_URL").expect("WEBSERVER_URL not set");

        let data_path = env::var("DATA_PATH").unwrap_or("./".to_string());
        let download_dir = format!("{}/downloads", data_path);
        let converted_dir = format!("{}/converted", data_path);

        let webserver_host = env::var("WEBSERVER_HOST").unwrap_or("0.0.0.0".to_string());
        let webserver_port = env::var("WEBSERVER_PORT")
            .unwrap_or("8080".to_string())
            .parse()
            .expect("WEBSERVER_PORT must be a valid u16");

        Self {
            discord_token,
            server_url,
            data_path,
            download_dir,
            converted_dir,
            webserver_host,
            webserver_port,
        }
    }

    pub fn download_path(&self) -> PathBuf {
        PathBuf::from(&self.download_dir)
    }

    pub fn converted_path(&self) -> PathBuf {
        PathBuf::from(&self.converted_dir)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}
