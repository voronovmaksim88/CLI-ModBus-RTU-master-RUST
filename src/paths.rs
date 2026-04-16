/// Функция получения пути к файлу настроек
pub fn get_settings_path() -> String {
    // В режиме разработки (cargo run) - в корне проекта
    // В режиме release (exe файл) - рядом с exe файлом
    if cfg!(debug_assertions) {
        "connect_settings.json".to_string()
    } else {
        match std::env::current_exe() {
            Ok(exe_path) => {
                if let Some(exe_dir) = exe_path.parent() {
                    exe_dir
                        .join("connect_settings.json")
                        .to_string_lossy()
                        .to_string()
                } else {
                    "connect_settings.json".to_string()
                }
            }
            Err(_) => "connect_settings.json".to_string(),
        }
    }
}

/// Функция получения пути к файлу регистров (CSV)
pub fn get_registers_path() -> String {
    if cfg!(debug_assertions) {
        "tags.csv".to_string()
    } else {
        match std::env::current_exe() {
            Ok(exe_path) => {
                if let Some(exe_dir) = exe_path.parent() {
                    exe_dir.join("tags.csv").to_string_lossy().to_string()
                } else {
                    "tags.csv".to_string()
                }
            }
            Err(_) => "tags.csv".to_string(),
        }
    }
}
