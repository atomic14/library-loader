use super::profile::Profile;
use super::consts::LL_CONFIG;
use super::error::{LLResult, LLError};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use clap::{self, load_yaml, crate_version};
use dirs;
use toml;

#[derive(Deserialize, Debug)]
struct ParseConfig {
    settings: Option<ParseSettings>,
    profile: Option<Profile>,
}

#[derive(Deserialize, Debug)]
struct ParseSettings {
    output_path: Option<String>,
    watch_path: Option<String>, // If set, enable watch mode
    format: Option<String> // If set, extract relevant files and place them in output_path
}

#[derive(Debug)]
pub struct Settings {
    pub output_path: String,
    pub watch_path: Option<String>,
    pub format: Option<String>
}

#[derive(Debug)]
pub struct Config {
    pub settings: Settings,
    pub profile: Profile,
    pub input: String,
    pub generate_config: bool,
    pub treat_input_as_id: bool
}

impl Config {

    pub fn load() -> Self {

        let conf: Self;

        let yml = load_yaml!("../cli.yml");
        let matches = clap::App::from(yml).version(crate_version!()).get_matches();

        let internal = Self::try_from_fs(matches.value_of("config"));

        // This needs to be refactored
        if internal.is_ok() {

            let int = internal.unwrap();

            let settings: Settings = match int.settings {
                Some(s) => {
                    Settings {
                        output_path: match s.output_path {
                            Some (v) => String::from(v),
                            None => Self::default().settings.output_path
                        },
                        watch_path: match s.watch_path {
                            Some(v) => Some(String::from(v)),
                            None => None
                        },
                        format: match s.format {
                            Some(v) => Some(String::from(v)),
                            None => None
                        },
                    }
                },
                None => Self::default().settings
            };

            let profile: Profile = match int.profile {
                Some(p) => p,
                None => {
                    match matches.is_present("generate") {
                        true => Profile::new("", ""),
                        false => Profile::prompt()
                    }
                }
            };

            conf = Self {
                settings: Settings {
                    output_path: match matches.value_of("output") {
                        Some(v) => String::from(v),
                        None => settings.output_path
                    },
                    watch_path: match matches.value_of("watch") {
                        Some(v) => Some(String::from(v)),
                        None => settings.watch_path
                    },
                    format: match matches.value_of("format") {
                        Some(v) => Some(String::from(v)),
                        None => settings.format
                    }
                },
                input: match matches.value_of("input") {
                    Some(v) => String::from(v),
                    None => Self::default().input
                },
                generate_config: matches.is_present("generate"),
                treat_input_as_id: matches.is_present("id"),
                profile: profile,
            }

        } else {

            conf = Self {
                settings: Settings {
                    output_path: match matches.value_of("output") {
                        Some(v) => String::from(v),
                        None => Self::default().settings.output_path
                    },
                    watch_path: match matches.value_of("watch") {
                        Some(v) => Some(String::from(v)),
                        None => Self::default().settings.watch_path
                    },
                    format: match matches.value_of("format") {
                        Some(v) => Some(String::from(v)),
                        None => Self::default().settings.format
                    },
                },
                input: match matches.value_of("input") {
                    Some(v) => String::from(v),
                    None => Self::default().input
                },
                generate_config: matches.is_present("generate"),
                treat_input_as_id: matches.is_present("id"),
                profile: match matches.is_present("generate") {
                    true => Profile::new("", ""),
                    false => Profile::prompt()
                }
            };

        }

        #[cfg(debug_assertions)]
        {
            println!("-- Debug info from {file}#{line} --", file = std::file!(), line = std::line!());
            println!("{:#?}", conf);
            println!("-- End debug info from {file}#{line} --", file = std::file!(), line = std::line!());
        }

        conf
    }

    fn try_from_fs(path_input: Option<&str>) -> LLResult<ParseConfig> {

        let path = match path_input {
            Some(p) => p,
            None => LL_CONFIG
        };

        let mut conf: Option<ParseConfig> = None;

        if PathBuf::from(path).exists() {
            let data = fs::read(path)?;
            conf = Some(toml::from_slice(&data)?);
        }

        // Don't bother checking global config is local is already set.
        if conf.is_none() {

            // Check home dir for LL Config.
            let home_path = match dirs::config_dir() {
                Some(hp) => Some(hp.join(LL_CONFIG)),
                None => None
            };

            if home_path.is_some() {
                let data = fs::read(home_path.unwrap())?;
                conf = Some(toml::from_slice(&data)?);
            }

        }

        match conf {
            Some(c) => Ok(c),
            None => Err(LLError::new(format!("{} not found", LL_CONFIG)))
        }

    }

    pub fn generate() -> LLResult<()> {

        let path = PathBuf::from(LL_CONFIG);

        if path.exists() {

            return Err(LLError::new(format!("{} already exists", LL_CONFIG)))

        }

        match fs::write(path, include_str!("../LibraryLoader.example.toml")) {
            Ok(v) => Ok(v),
            Err(e) => Err(LLError::new(format!("{}", e)))
        }

    }

}

impl Default for Config {

    fn default() -> Self {

        let profile = Profile::new("", "");

        Self {
            settings: Settings {
                output_path: String::from("download"),
                watch_path: None,
                format: None
            },
            profile: profile,
            input: String::new(),
            generate_config: false,
            treat_input_as_id: false
        }

    }

}