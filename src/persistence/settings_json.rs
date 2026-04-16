use colored::*;
use std::fs;
use std::io;

use crate::config::model::{Config, ConnectionSettings, Metadata, default_connection_mode, default_tcp_host, default_tcp_port};
use crate::paths::get_settings_path;

/// Функция создания настроек подключения по умолчанию
pub fn create_default_settings() -> ConnectionSettings {
    ConnectionSettings {
        port: "COM1".to_string(),
        device_address: 1,
        baud_rate: 9600,
        parity: "None".to_string(),
        stop_bits: 1,
        mode: default_connection_mode(),
        tcp_host: default_tcp_host(),
        tcp_port: default_tcp_port(),
    }
}

/// Функция загрузки настроек из JSON файла
pub fn load_settings() -> io::Result<Config> {
    let settings_path = get_settings_path();
    let file_content = fs::read_to_string(&settings_path)?;
    let config: Config = serde_json::from_str(&file_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(config)
}

/// Функция сохранения настроек в JSON файл
pub fn save_settings(connection: ConnectionSettings) -> io::Result<()> {
    let metadata = Metadata {
        last_updated: chrono::Utc::now().to_rfc3339(),
        version: "1.0".to_string(),
        description: "Настройки подключения для Modbus RTU и Modbus TCP".to_string(),
    };

    let config = Config {
        connection,
        metadata,
    };

    let json_content = serde_json::to_string_pretty(&config)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let settings_path = get_settings_path();
    fs::write(&settings_path, json_content)?;
    Ok(())
}

/// Функция инициализации конфигурации при первом запуске
pub fn initialize_config_if_needed() -> io::Result<()> {
    let settings_path = get_settings_path();

    if !std::path::Path::new(&settings_path).exists() {
        println!("{}", "Файл настроек не найден. Создаём конфигурацию по умолчанию...".yellow());

        let default_connection = create_default_settings();
        match save_settings(default_connection) {
            Ok(()) => {
                println!("{}", "Файл connect_settings.json создан с настройками по умолчанию".green());
                println!("\n{}", "Настройки по умолчанию:".cyan());
                println!("  • COM-порт: COM1");
                println!("  • Адрес устройства: 1");
                println!("  • Скорость: 9600 бод");
                println!("  • Четность: None");
                println!("  • Стоп-биты: 1");
                println!("  • Режим: RTU");
                println!("  • TCP хост: 127.0.0.1");
                println!("  • TCP порт: 502");
                println!();
            }
            Err(e) => {
                eprintln!("{}", format!("Ошибка создания файла настроек: {}", e).red());
                return Err(e);
            }
        }
    }

    Ok(())
}
