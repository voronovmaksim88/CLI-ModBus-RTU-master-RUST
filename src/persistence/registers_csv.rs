use colored::*;
use std::fs;
use std::io;

use crate::config::model::{Metadata, RegisterConfig, RegistersConfig};
use crate::paths::get_registers_path;

/// Функция загрузки конфигурации регистров из CSV файла
pub fn load_registers() -> io::Result<RegistersConfig> {
    let registers_path = get_registers_path();
    let file = fs::File::open(&registers_path)?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(file);

    let mut registers: Vec<RegisterConfig> = Vec::new();
    for record in reader.deserialize::<RegisterConfig>() {
        match record {
            Ok(register) => registers.push(register),
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    let metadata = Metadata {
        last_updated: chrono::Utc::now().to_rfc3339(),
        version: "csv-1.0".to_string(),
        description: "Конфигурация регистров из CSV (tags.csv)".to_string(),
    };

    Ok(RegistersConfig {
        registers,
        metadata,
    })
}

/// Загрузка регистров с единообразным уведомлением об ошибке
pub fn load_registers_or_warn() -> Option<RegistersConfig> {
    match load_registers() {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("{}", format!("Не удалось загрузить регистры: {}", e).red());
            println!(
                "{}",
                "Убедитесь, что файл tags.csv существует и корректен".yellow()
            );
            None
        }
    }
}

/// Сохранение регистров обратно в CSV (tags.csv)
pub fn save_registers_to_csv(registers: &[RegisterConfig]) -> io::Result<()> {
    let path = get_registers_path();
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(&path)
        .map_err(io::Error::other)?;

    writer
        .write_record([
            "name",
            "description",
            "address",
            "var_type",
            "modbus_type",
            "enabled",
        ])
        .map_err(io::Error::other)?;

    for reg in registers {
        writer
            .write_record([
                reg.name.as_str(),
                reg.description.as_str(),
                &reg.address.to_string(),
                reg.var_type.as_str(),
                reg.modbus_type.as_str(),
                if reg.enabled { "true" } else { "false" },
            ])
            .map_err(io::Error::other)?;
    }

    writer.flush().map_err(io::Error::other)?;
    Ok(())
}

/// Функция инициализации файла регистров при первом запуске
pub fn initialize_registers_if_needed() -> io::Result<()> {
    let registers_path = get_registers_path();

    if !std::path::Path::new(&registers_path).exists() {
        println!("{}", "Файл регистров не найден. Создаём пустой файл tags.csv...".yellow());
        let empty_registers: Vec<RegisterConfig> = Vec::new();

        match save_registers_to_csv(&empty_registers) {
            Ok(()) => {
                println!("{}", "Файл tags.csv создан (пустой список регистров)".green());
                println!("{}", "Вы можете добавить регистры через меню 'Регистры' -> 'Добавить регистр'".bright_black());
                println!();
            }
            Err(e) => {
                eprintln!("{}", format!("Ошибка создания файла регистров: {}", e).red());
                return Err(e);
            }
        }
    }

    Ok(())
}
