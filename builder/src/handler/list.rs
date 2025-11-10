use std::{
    fmt::{Display, Formatter},
    fs::read_dir,
    path::PathBuf,
};

use signer::SigningKeyPair;
use tabled::{settings::Style, Table, Tabled};
use tracing::error;

use crate::args::list::{ListArgs, ListCommands};

#[derive(Tabled)]
struct KeyRow {
    label:      String,
    public_key: String,
    created_at: String,
}

#[derive(Tabled)]
struct PluginRow {
    name:       String,
    version:    String,
    origin:     PluginOrigin,
    created_at: String,
}

enum PluginOrigin {
    SourceCode,
    CompiledRelease,
    CompiledDebug,
    Compiled,
}

impl Display for PluginOrigin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginOrigin::SourceCode => {
                write!(f, "Source Code")
            },
            PluginOrigin::CompiledRelease => {
                write!(f, "Compiled in Release Mode")
            },
            PluginOrigin::CompiledDebug => {
                write!(f, "Compiled in Debug Mode")
            },
            PluginOrigin::Compiled => {
                write!(f, "Compiled")
            },
        }
    }
}

pub fn handle(args: ListArgs, is_json: bool) {
    if !is_json {
        handle_text(args)
    }
    else {
        handle_json(args)
    }
}

fn handle_json(args: ListArgs) {}

fn handle_text(args: ListArgs) {
    match args.command {
        ListCommands::Keys {
            storage,
        } => {
            if !storage.exists() {
                error!("No keys found. Storage directory does not exist.");
                return;
            }

            let entries = match read_dir(storage) {
                Ok(entries) => entries,
                Err(e) => {
                    error!("Failed to read storage directory: {}", e);
                    return;
                },
            };

            let mut keys = Vec::<KeyRow>::new();

            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("key")) {
                            let private_key = match std::fs::read_to_string(&path) {
                                Ok(content) => content,
                                Err(e) => {
                                    error!("Failed to read key file {:?}: {}", path, e);
                                    continue;
                                },
                            };

                            let mut keypair_failed = false;
                            let keypair = match SigningKeyPair::from_private_key_hex(&private_key) {
                                Ok(kp) => kp,
                                Err(_) => {
                                    keypair_failed = true;
                                    continue;
                                },
                            };

                            keys.push(KeyRow {
                                label:      path.file_stem().unwrap().to_string_lossy().to_string(),
                                public_key: if !keypair_failed {
                                    keypair.public_key_hex()
                                }
                                else {
                                    "Unable to parse key".to_owned()
                                },
                                created_at: path
                                    .metadata()
                                    .and_then(|meta| meta.created())
                                    .map(|time| {
                                        let datetime: chrono::DateTime<chrono::Utc> = time.into();
                                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                                    })
                                    .unwrap_or_else(|_| "Unknown".to_owned()),
                            });
                        }
                    },
                    Err(e) => {
                        error!("Failed to read an entry in storage directory: {}", e);
                    },
                }
            }

            let mut table = Table::new(keys);
            table.with(Style::modern());
            println!("{}", table);
        },
        ListCommands::Plugins {
            plugins_dir,
            with_cargo_build,
            with_cargo_debug,
        } => {
            if !plugins_dir.exists() {
                error!("Plugins directory does not exist.");
                return;
            }

            let entries = match read_dir(plugins_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    error!("Failed to read plugins directory: {}", e);
                    return;
                },
            };

            let mut plugins = Vec::<PluginRow>::new();

            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();

                        // plugins are compiled
                        if path.is_file() && is_plugin(&path) {
                            plugins.push(parse_compiled_file(&path, PluginOrigin::Compiled));
                        }
                        // plugins are source code
                        else if path.is_dir() && is_plugin(&path) {
                            if let Some(plugin_row) = parse_from_code(&path) {
                                plugins.push(plugin_row);
                            }
                        }
                    },
                    Err(e) => {
                        error!("Failed to read an entry in plugins directory: {}", e);
                    },
                }
            }

            if with_cargo_build {
                let cargo_release_dir = PathBuf::from("./target/release/");
                if cargo_release_dir.exists() {
                    let cargo_entries = match read_dir(cargo_release_dir) {
                        Ok(entries) => entries,
                        Err(e) => {
                            error!("Failed to read cargo release directory: {}", e);
                            return;
                        },
                    };

                    for entry in cargo_entries {
                        match entry {
                            Ok(entry) => {
                                let path = entry.path();
                                if path.is_file() && is_plugin(&path) {
                                    plugins.push(parse_compiled_file(&path, PluginOrigin::CompiledRelease));
                                }
                            },
                            Err(e) => {
                                error!("Failed to read an entry in cargo release directory: {}", e);
                            },
                        }
                    }
                }
            }

            if with_cargo_debug {
                let cargo_debug_dir = PathBuf::from("./target/debug/");
                if cargo_debug_dir.exists() {
                    let cargo_entries = match read_dir(cargo_debug_dir) {
                        Ok(entries) => entries,
                        Err(e) => {
                            error!("Failed to read cargo debug directory: {}", e);
                            return;
                        },
                    };

                    for entry in cargo_entries {
                        match entry {
                            Ok(entry) => {
                                let path = entry.path();
                                if path.is_file() && is_plugin(&path) {
                                    plugins.push(parse_compiled_file(&path, PluginOrigin::CompiledDebug));
                                }
                            },
                            Err(e) => {
                                error!("Failed to read an entry in cargo debug directory: {}", e);
                            },
                        }
                    }
                }
            }
            let mut table = Table::new(plugins);
            table.with(Style::modern());
            println!("{}", table);
        },
    }
}

fn is_plugin(path: &PathBuf) -> bool {
    let file_name = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return false,
    };

    !file_name.starts_with("plugin-worker") && (
        (file_name.starts_with("plugin-") && !file_name.ends_with(".d")) ||
        file_name.ends_with("-plugin")
    )
}

fn parse_compiled_file(path: &PathBuf, from_origin: PluginOrigin) -> PluginRow {
    let metadata = path.metadata();
    let created_at = metadata
        .and_then(|meta| meta.created())
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Utc> = time.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|_| "Unknown".to_owned());

    let plugin_name = path.file_stem().unwrap().to_string_lossy().to_string();
    let version = plugin_name
        .split("-")
        .collect::<Vec<&str>>()
        .last()
        .unwrap_or(&"Unknown")
        .to_string();

    PluginRow {
        name: plugin_name.replace("plugin-", "").replace("-plugin", "").replace(format!("-{}", version).as_str(), ""),
        version: if version != "Unknown" { format!("v{}", version) } else { version },
        origin: from_origin,
        created_at,
    }
}

fn parse_from_code(path: &PathBuf) -> Option<PluginRow> {
    let original_path = path.file_name()?.to_string_lossy().to_string();
    let cargo_toml_path = path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let cargo_toml_content = match std::fs::read_to_string(&cargo_toml_path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read Cargo.toml for plugin {:?}: {}", path, e);
                return None;
            },
        };
        let mut version = cargo_toml_content
            .lines()
            .find(|line| line.starts_with("version"))
            .and_then(|line| line.split('=').nth(1))
            .map(|v| v.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        if version != "Unknown" {
            version = format!("v{}", version);
        }

        let metadata = path.metadata();
        let created_at = metadata
            .and_then(|meta| meta.created())
            .map(|time| {
                let datetime: chrono::DateTime<chrono::Utc> = time.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|_| "Unknown".to_owned());

        return Some(PluginRow {
            name: original_path.replace("plugin-", "").replace("-plugin", ""),
            version,
            origin: PluginOrigin::SourceCode,
            created_at,
        });
    }

    None
}
