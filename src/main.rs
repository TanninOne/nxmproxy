#![windows_subsystem = "windows"]
use config_file::{Config};
use directories::BaseDirs;
use win32::{get_protocol_handler, parse_commandline, set_protocol_handler};
use std::ffi::OsStr;
use std::fs::{DirBuilder};
use std::path::{Path, PathBuf};
use std::{process::Command};
use std::{
    io::{Error},
    process::Stdio,
};
use structopt::StructOpt;
use url::Url;

use crate::pipe::try_write_pipe;
use crate::win32::spawn_elevated;

mod win32;
mod config_file;
mod pipe;
mod logger;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "nxmproxy",
    about = "Forward Nexus Mods nxm links to the correct manager based on game."
)]
enum Options {
    /// download a url
    Url {
        /// the url to download
        url: String,
    },
    /// register a mod manager
    Register {
        /// name of the manager to register
        manager: String,
        /// the command to use to start the manager and to queue downloads
        command: String,
    },
    /// unregister a mod manager
    Deregister {
        /// name of the manager
        manager: String,
    },
    /// assign a mod manager to handle links for a game. If there is already a manager assigned
    /// to this game it gets overridden
    Assign {
        /// name of the manager
        manager: String,
        /// the game to manage (has to correspond to the id used Nexus Mods)
        game: String,
    },
    /// set up to use a named pipe to deliver urls to the manager if it's already running
    /// (and listening on this pipe of course)
    Pipe {
        /// name of the manager
        manager: String,
        /// name of the pipe (not the entire path, only the name)
        pipe: String,
    },
    /// Install nxmproxy as the handler for nxm links. This replaces the old handler
    Install,
    /// Tests wether nxmproxy is set up as the handler for nxm links
    Test,
}


/// create a directory (structure) if necessary
fn ensure_dir(str: &str) -> Result<(), Error> {
    return DirBuilder::new().recursive(true).create(str);
}

/// handle download url with the appropriate manager
fn download(config: &Config, url: &str) -> Result<(), String> {
    let parsed_url = Url::parse(url).expect("Failed to parse url");
    if parsed_url.scheme() != "nxm" {
        return Err("Not an nxm url".to_string());
    }

    let game = parsed_url.host_str().expect("Invalid url");

    let manager = config.resolve(game).expect("Failed to find manager for game");

    logger::info(format!("downloading url: {}, game: {}, manager: {}",
        url, game, manager));

    if config.pipes.contains_key(&manager) {
        logger::info(format!("trying pipe: {}", config.pipes[&manager]));
        if try_write_pipe(&config.pipes[&manager], url) {
            return Ok(());
        } else {
            // this may not be a problem, the manager may just not be running yet
            logger::info(format!("pipe write failed"));
        }
    }

    let command_line = config.managers[&manager].to_string().replace("%1", url);
    let (exe, args) = parse_commandline(&command_line)
        .expect("Failed to parse command line");

    Command::new(exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn manager");

    Ok(())
}

/// set up nxmproxy as the handler for nxm links
fn install() -> Result<(), Error> {
    let command = std::env::current_exe()
        .and_then(|path| { Ok(String::from(path.as_path().to_str().unwrap())) })?;

    logger::info(format!("registering as nxm handler: {}", command));
    let res =
        set_protocol_handler("nxm", format!("\"{}\" url \"%1\"", command).as_str());

    if let Err(err) = res {
        if err.raw_os_error() == Some(5) {
            logger::info("registering requires elevation");
            let self_exe = std::env::current_exe()
                .expect("failed to determine current exe");

            return spawn_elevated(self_exe.to_str().unwrap(), vec!["install"]);
        } else {
            Err(err)
        }
    } else {
        Ok(())
    }
}

/// check whether nxmproxy (any instance) is assigned as the nxm handler
fn test_installed() -> Result<bool, Error> {
    let handler = get_protocol_handler("nxm");
    match handler {
        Ok(commandline) => {
            let (exe_path, _args) = parse_commandline(commandline.as_str())
                .expect("Failed to parse command line");
            let exe_name = Path::new(&exe_path).file_name().unwrap();
            Ok(exe_name == OsStr::new("nxmproxy.exe"))
        },
        Err(e) => {
            Err(e)
        }
    }
}

fn applocal_path() -> PathBuf {
    let base_dirs = BaseDirs::new().expect("Failed to query operating system default directories");
    let config_path_buf = base_dirs.data_local_dir().join("nxmproxy");
    return config_path_buf;
}

fn main_impl() -> Result<i32, String> {
    let opt = Options::from_args();

    let config_path_buf = applocal_path();
    let config_path = config_path_buf.as_path();

    ensure_dir(config_path.to_str().expect("No config path?"))
        .expect("Failed to create project directory");

    let mut config = Config::read(config_path).expect("Failed to read config");

    return match opt {
        Options::Url { url } => download(&config, &url).and(Ok(0)),
        Options::Assign { manager, game } => {
            config.assign(&manager, &game)?;
            return config.write_config(config_path).and(Ok(0));
        }
        Options::Register { manager, command } => {
            config.register(&manager, &command)?;
            return config.write_config(config_path).and(Ok(0));
        }
        Options::Pipe { manager, pipe } => {
            config.register_pipe(&manager, &pipe)?;
            return config.write_config(config_path).and(Ok(0));
        }
        Options::Deregister { manager } => {
            config.deregister(&manager)?;
            return config.write_config(config_path).and(Ok(0));
        }
        Options::Install {} => {
            match install() {
                Ok(()) => Ok(0),
                Err(e) => Err(e.to_string()),
            }
        }
        Options::Test {} => {
            match test_installed() {
                Ok(is_installed) => {
                    println!("is installed: {}", is_installed);
                    Ok(if is_installed { 0 } else { 1 })
                },
                Err(e) => Err(e.to_string()),
            }
        }
    }
}

fn main() -> Result<(), String> {
    let config_path_buf = applocal_path();
    let config_path = config_path_buf.as_path();
    logger::set_file(config_path.join("nxm.log").as_path().to_str().unwrap());

    match main_impl() {
        Ok(res) => {
            std::process::exit(res);
        }
        Err(e) => {
            logger::error(format!("Failed to process commandline: {}", e));
            std::process::exit(1);
        }
    }
}

