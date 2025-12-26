use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

const DEFAULT_POSTGRES_TAG: &str = "18-alpine";
const DEFAULT_MYSQL_TAG: &str = "8.0";
const DEFAULT_MONGODB_TAG: &str = "8";
const DEFAULT_REDIS_TAG: &str = "8-alpine";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            list_databases,
            start_database,
            stop_database,
            get_database_status,
            set_volume_path,
            get_volume_path,
            set_image_tag,
            get_image_tag,
            get_container_logs,
            generate_connection_strings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize, Clone)]
struct DatabaseInfo {
    name: String,
    status: String,
    port: u16,
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CommandResult {
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Config {
    volume_paths: HashMap<String, String>,
    #[serde(default)]
    image_tags: HashMap<String, String>,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("dbrunner");
    fs::create_dir_all(&path).ok();
    path.push("config.json");
    path
}

fn load_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

fn save_config(config: &Config) -> Result<(), String> {
    let path = get_config_path();
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_config() -> Config {
    let mut config_lock = CONFIG.lock().unwrap();
    if config_lock.is_none() {
        *config_lock = Some(load_config());
    }
    config_lock.clone().unwrap()
}

fn get_database_image(db_name: &str, config: &Config) -> String {
    let (base_image, default_tag) = match db_name {
        "postgresql" => ("postgres", DEFAULT_POSTGRES_TAG),
        "mysql" => ("mysql", DEFAULT_MYSQL_TAG),
        "mongodb" => ("mongo", DEFAULT_MONGODB_TAG),
        "redis" => ("redis", DEFAULT_REDIS_TAG),
        _ => ("", ""),
    };

    let tag = config
        .image_tags
        .get(db_name)
        .map(|s| s.as_str())
        .unwrap_or(default_tag);

    format!("{}:{}", base_image, tag)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn list_databases() -> Vec<DatabaseInfo> {
    let config = get_config();

    vec![
        DatabaseInfo {
            name: "PostgreSQL".to_string(),
            status: "stopped".to_string(),
            port: 5432,
            image: get_database_image("postgresql", &config),
            volume_path: config.volume_paths.get("postgresql").cloned(),
        },
        DatabaseInfo {
            name: "MySQL".to_string(),
            status: "stopped".to_string(),
            port: 3306,
            image: get_database_image("mysql", &config),
            volume_path: config.volume_paths.get("mysql").cloned(),
        },
        DatabaseInfo {
            name: "MongoDB".to_string(),
            status: "stopped".to_string(),
            port: 27017,
            image: get_database_image("mongodb", &config),
            volume_path: config.volume_paths.get("mongodb").cloned(),
        },
        DatabaseInfo {
            name: "Redis".to_string(),
            status: "stopped".to_string(),
            port: 6379,
            image: get_database_image("redis", &config),
            volume_path: config.volume_paths.get("redis").cloned(),
        },
    ]
}

#[tauri::command]
fn set_volume_path(db_name: String, path: String) -> CommandResult {
    let mut config_lock = CONFIG.lock().unwrap();
    let mut config = config_lock.clone().unwrap_or_default();

    let db_key = db_name.to_lowercase();

    // Validate the path exists
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return CommandResult {
            success: false,
            message: format!("Path does not exist: {}", path),
        };
    }

    config.volume_paths.insert(db_key, path);

    match save_config(&config) {
        Ok(_) => {
            *config_lock = Some(config);
            CommandResult {
                success: true,
                message: format!("Volume path set for {}", db_name),
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to save config: {}", e),
        },
    }
}

#[tauri::command]
fn get_volume_path(db_name: String) -> Option<String> {
    let config = get_config();
    config.volume_paths.get(&db_name.to_lowercase()).cloned()
}

#[tauri::command]
fn set_image_tag(db_name: String, tag: String) -> CommandResult {
    let mut config_lock = CONFIG.lock().unwrap();
    let mut config = config_lock.clone().unwrap_or_default();

    let db_key = db_name.to_lowercase();
    let trimmed_tag = tag.trim();

    // If tag is empty, remove from config to use default
    if trimmed_tag.is_empty() {
        config.image_tags.remove(&db_key);
        match save_config(&config) {
            Ok(_) => {
                *config_lock = Some(config);
                return CommandResult {
                    success: true,
                    message: format!("Reset {} to default image tag", db_name),
                };
            }
            Err(e) => {
                return CommandResult {
                    success: false,
                    message: format!("Failed to save config: {}", e),
                }
            }
        }
    }

    // Validation: basic tag format (no colons, no slashes, reasonable length)
    if trimmed_tag.contains(':') || trimmed_tag.contains('/') || trimmed_tag.len() > 100 {
        return CommandResult {
            success: false,
            message: "Invalid tag format. Tag should be version/variant only (e.g., '16-alpine')"
                .to_string(),
        };
    }

    config.image_tags.insert(db_key, trimmed_tag.to_string());

    match save_config(&config) {
        Ok(_) => {
            *config_lock = Some(config);
            CommandResult {
                success: true,
                message: format!("Image tag set for {}", db_name),
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to save config: {}", e),
        },
    }
}

#[tauri::command]
fn get_image_tag(db_name: String) -> Option<String> {
    let config = get_config();
    config.image_tags.get(&db_name.to_lowercase()).cloned()
}

fn generate_docker_compose(
    db_name: &str,
    custom_path: Option<&str>,
    config: &Config,
) -> Option<String> {
    let image = get_database_image(db_name, config);

    let (container_name, port, default_volume, env_vars, health_check) = match db_name {
        "postgresql" => (
            "dbrunner-postgres",
            "5432:5432",
            "/var/lib/postgresql/data",
            vec![
                "POSTGRES_USER: postgres",
                "POSTGRES_PASSWORD: postgres",
                "POSTGRES_DB: devdb",
            ],
            r#"test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5"#,
        ),
        "mysql" => (
            "dbrunner-mysql",
            "3306:3306",
            "/var/lib/mysql",
            vec![
                "MYSQL_ROOT_PASSWORD: root",
                "MYSQL_DATABASE: devdb",
                "MYSQL_USER: mysql",
                "MYSQL_PASSWORD: mysql",
            ],
            r#"test: ["CMD", "mysqladmin", "ping", "-h", "localhost", "-u", "root", "-proot"]
      interval: 10s
      timeout: 5s
      retries: 5"#,
        ),
        "mongodb" => (
            "dbrunner-mongodb",
            "27017:27017",
            "/data/db",
            vec![
                "MONGO_INITDB_ROOT_USERNAME: admin",
                "MONGO_INITDB_ROOT_PASSWORD: admin",
                "MONGO_INITDB_DATABASE: devdb",
            ],
            r#"test: ["CMD", "mongosh", "--eval", "db.adminCommand('ping')"]
      interval: 10s
      timeout: 5s
      retries: 5"#,
        ),
        "redis" => (
            "dbrunner-redis",
            "6379:6379",
            "/data",
            vec![],
            r#"test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5"#,
        ),
        _ => return None,
    };

    let volume_name = match db_name {
        "postgresql" => "postgres",
        "mysql" => "mysql",
        "mongodb" => "mongodb",
        "redis" => "redis",
        _ => db_name,
    };

    let volume_line = if let Some(path) = custom_path {
        format!("      - {}:{}", path, default_volume)
    } else {
        format!("      - {}_data:{}", volume_name, default_volume)
    };

    let volumes_section = if custom_path.is_none() {
        format!("\nvolumes:\n  {}_data:\n    driver: local", volume_name)
    } else {
        String::new()
    };

    let env_section = if env_vars.is_empty() {
        String::new()
    } else {
        format!(
            "    environment:\n{}",
            env_vars
                .iter()
                .map(|e| format!("      {}", e))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    Some(format!(
        r#"version: '3.8'

services:
  {}:
    image: {}
    container_name: {}
{}
    ports:
      - "{}"
    volumes:
{}
    restart: unless-stopped
    healthcheck:
      {}
{}"#,
        db_name,
        image,
        container_name,
        env_section,
        port,
        volume_line,
        health_check,
        volumes_section
    ))
}

#[tauri::command]
fn start_database(db_name: String) -> CommandResult {
    let db_key = db_name.to_lowercase();
    let config = get_config();
    let custom_path = config.volume_paths.get(&db_key);

    // Generate docker-compose content
    let compose_content =
        match generate_docker_compose(&db_key, custom_path.map(|s| s.as_str()), &config) {
            Some(content) => content,
            None => {
                return CommandResult {
                    success: false,
                    message: format!("Unknown database: {}", db_name),
                }
            }
        };

    // Write to a temporary file
    let temp_dir = std::env::temp_dir();
    let compose_file = temp_dir.join(format!("dbrunner-{}.yml", db_key));

    if let Err(e) = fs::write(&compose_file, compose_content) {
        return CommandResult {
            success: false,
            message: format!("Failed to create compose file: {}", e),
        };
    }

    let output = Command::new("docker")
        .args(["compose", "-f", compose_file.to_str().unwrap(), "up", "-d"])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                CommandResult {
                    success: true,
                    message: format!("{} started successfully", db_name),
                }
            } else {
                CommandResult {
                    success: false,
                    message: String::from_utf8_lossy(&result.stderr).to_string(),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to start database: {}", e),
        },
    }
}

#[tauri::command]
fn stop_database(db_name: String) -> CommandResult {
    let db_key = db_name.to_lowercase();

    // Use the same temporary compose file path
    let temp_dir = std::env::temp_dir();
    let compose_file = temp_dir.join(format!("dbrunner-{}.yml", db_key));

    // Check if the compose file exists, if not it might be using the old template
    let file_to_use = if compose_file.exists() {
        compose_file.to_str().unwrap().to_string()
    } else {
        // Fallback to old template files
        match get_template_file(&db_key) {
            Some(f) => f.to_string(),
            None => {
                return CommandResult {
                    success: false,
                    message: format!("Unknown database: {}", db_name),
                }
            }
        }
    };

    let output = Command::new("docker")
        .args(["compose", "-f", &file_to_use, "down"])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                // Clean up the temporary compose file if it exists
                if compose_file.exists() {
                    fs::remove_file(&compose_file).ok();
                }
                CommandResult {
                    success: true,
                    message: format!("{} stopped successfully", db_name),
                }
            } else {
                CommandResult {
                    success: false,
                    message: String::from_utf8_lossy(&result.stderr).to_string(),
                }
            }
        }
        Err(e) => CommandResult {
            success: false,
            message: format!("Failed to stop database: {}", e),
        },
    }
}

fn get_template_file(db_name: &str) -> Option<&'static str> {
    match db_name.to_lowercase().as_str() {
        "postgresql" => Some("docker-templates/postgres.yml"),
        "mysql" => Some("docker-templates/mysql.yml"),
        "mongodb" => Some("docker-templates/mongodb.yml"),
        "redis" => Some("docker-templates/redis.yml"),
        _ => None,
    }
}

#[tauri::command]
fn get_database_status(db_name: String) -> String {
    let container_name = match db_name.to_lowercase().as_str() {
        "postgresql" => "dbrunner-postgres",
        "mysql" => "dbrunner-mysql",
        "mongodb" => "dbrunner-mongodb",
        "redis" => "dbrunner-redis",
        _ => return "unknown".to_string(),
    };

    let output = Command::new("docker")
        .args([
            "ps",
            "--filter",
            &format!("name={}", container_name),
            "--format",
            "{{.Status}}",
        ])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let status_output = String::from_utf8_lossy(&result.stdout);
                if status_output.trim().is_empty() {
                    "stopped".to_string()
                } else if status_output.contains("Up") {
                    "running".to_string()
                } else {
                    "stopped".to_string()
                }
            } else {
                "error".to_string()
            }
        }
        Err(_) => "error".to_string(),
    }
}

#[tauri::command]
fn get_container_logs(db_name: String, tail_lines: Option<usize>) -> Result<String, String> {
    let container_name = match db_name.to_lowercase().as_str() {
        "postgresql" => "dbrunner-postgres",
        "mysql" => "dbrunner-mysql",
        "mongodb" => "dbrunner-mongodb",
        "redis" => "dbrunner-redis",
        _ => return Err("Unknown database".to_string()),
    };

    let lines = tail_lines.unwrap_or(100).to_string();

    let output = Command::new("docker")
        .args(["logs", "--tail", &lines, container_name])
        .output()
        .map_err(|e| format!("Failed to get logs: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Combine stdout and stderr as Docker logs can output to both
        let mut combined = String::new();
        if !stderr.is_empty() {
            combined.push_str(&stderr);
        }
        if !stdout.is_empty() {
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&stdout);
        }

        if combined.is_empty() {
            Ok("No logs available".to_string())
        } else {
            Ok(combined)
        }
    } else {
        Err(format!(
            "Container not running or not found: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[tauri::command]
fn generate_connection_strings(
    db_name: String,
    port: u16,
) -> Result<HashMap<String, String>, String> {
    let db_key = db_name.to_lowercase();
    let mut result = HashMap::new();

    match db_key.as_str() {
        "postgresql" => {
            result.insert(
                "standard_uri".to_string(),
                format!("postgresql://postgres:postgres@localhost:{}/devdb", port),
            );
            result.insert(
                "jdbc".to_string(),
                format!("jdbc:postgresql://localhost:{}/devdb", port),
            );
            result.insert("host".to_string(), "localhost".to_string());
            result.insert("port".to_string(), port.to_string());
            result.insert("user".to_string(), "postgres".to_string());
            result.insert("password".to_string(), "postgres".to_string());
            result.insert("database".to_string(), "devdb".to_string());
        }
        "mysql" => {
            result.insert(
                "standard_uri".to_string(),
                format!("mysql://root:root@localhost:{}/devdb", port),
            );
            result.insert(
                "jdbc".to_string(),
                format!("jdbc:mysql://localhost:{}/devdb", port),
            );
            result.insert("host".to_string(), "localhost".to_string());
            result.insert("port".to_string(), port.to_string());
            result.insert("user".to_string(), "root".to_string());
            result.insert("password".to_string(), "root".to_string());
            result.insert("database".to_string(), "devdb".to_string());
        }
        "mongodb" => {
            result.insert(
                "standard_uri".to_string(),
                format!("mongodb://admin:admin@localhost:{}/devdb", port),
            );
            result.insert(
                "jdbc".to_string(),
                "N/A (MongoDB uses native driver)".to_string(),
            );
            result.insert("host".to_string(), "localhost".to_string());
            result.insert("port".to_string(), port.to_string());
            result.insert("user".to_string(), "admin".to_string());
            result.insert("password".to_string(), "admin".to_string());
            result.insert("database".to_string(), "devdb".to_string());
        }
        "redis" => {
            result.insert(
                "standard_uri".to_string(),
                format!("redis://localhost:{}", port),
            );
            result.insert(
                "jdbc".to_string(),
                "N/A (Redis uses native driver)".to_string(),
            );
            result.insert("host".to_string(), "localhost".to_string());
            result.insert("port".to_string(), port.to_string());
            result.insert("user".to_string(), "N/A".to_string());
            result.insert("password".to_string(), "N/A".to_string());
            result.insert("database".to_string(), "0 (default)".to_string());
        }
        _ => return Err(format!("Unknown database: {}", db_name)),
    }

    Ok(result)
}
