use std::{collections::HashMap, fs::{File, read_to_string, rename}, io::{Error, ErrorKind, Write}, path::Path};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub games: HashMap<String, String>,
    pub managers: HashMap<String, String>,
    pub pipes: HashMap<String, String>,
}

static EMPTY_CONFIG: &str = r#"
[games]

[managers]

[pipes]
"#;

impl Config {
    /// read config from disc
    pub fn read(config_path: &Path) -> Result<Config, Error> {
        let config_data = match read_to_string(config_path.join("config.toml")) {
            Ok(s) => s,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => EMPTY_CONFIG.to_string(),
                other_error => {
                    panic!("Failed to read config: {:?}", other_error)
                }
            },
        };
        let config: Config = toml::from_str(&config_data).expect("Failed to parse game config");

        Ok(config)
    }

    /// write configuration to disc
    pub fn write_config(&self, config_path: &Path) -> Result<(), String> {
        let updated_config = toml::to_string(self).expect("Failed to update config");

        let config_file_path = config_path
            .join("config.toml")
            .to_str()
            .unwrap()
            .to_string();
        let config_temp_path = config_file_path.clone() + ".tmp";

        let mut buffer = File::create(&config_temp_path).expect("Failed to create config file");

        buffer
            .write_all(updated_config.as_bytes())
            .expect("Failed to write config file");

        rename(&config_temp_path, &config_file_path).expect("Failed to apply config file changes");

        Ok(())
    }

    /// assign a game to be handled by a manager
    pub fn assign(&mut self, manager: &str, game: &str) -> Result<(), String> {
        self.games.insert(game.to_string(), manager.to_string());
        Ok(())
    }

    /// register a manager
    pub fn register(&mut self, manager: &str, command: &str) -> Result<(), String> {
        self
            .managers
            .insert(manager.to_string(), command.to_string());
        Ok(())
    }

    /// deregister a manager
    pub fn deregister(&mut self, manager: &str) -> Result<(), String> {
        self.managers.remove(manager);
        self.pipes.remove(manager);
        Ok(())
    }

    /// register a named pipe to send urls to
    pub fn register_pipe(&mut self, manager: &str, pipe: &str) -> Result<(), String> {
        if !self.managers.contains_key(manager) {
            return Err(format!(r#"Manager {} is not registered"#, manager));
        }
        self.pipes.insert(manager.to_string(), pipe.to_string());
        Ok(())
    }

    pub fn resolve(&self, game: &str) -> Result<String, String> {
        let manager: String;

        if self.games.contains_key(game) {
            manager = self.games[game].to_string();
        } else if self.games.contains_key("_") {
            manager = self.games["_"].to_string();
        } else {
            return Err(format!("No manager for game {}", game));
        }

        return Ok(manager);
    }
}
