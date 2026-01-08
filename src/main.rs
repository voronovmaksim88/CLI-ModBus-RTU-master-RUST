mod add_register;
mod scan_available_ports;
mod sort_registers;
use add_register::add_register;
use colored::*;
use scan_available_ports::scan_available_ports;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::time::Duration;
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

#[cfg(windows)]
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
#[cfg(windows)]
use winapi::um::processenv::GetStdHandle;
#[cfg(windows)]
use winapi::um::winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
#[cfg(windows)]
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

/// Доступные скорости передачи данных для RS-485 (в бодах)
const AVAILABLE_BAUD_RATES: (u32, u32, u32, u32, u32, u32, u32) =
    (2400, 4800, 9600, 19200, 38400, 57600, 115200);

/// Доступные варианты четности для RS-485
const PARITY_OPTIONS: (&str, &str, &str) = ("None", "Even", "Odd");

/// Доступные варианты стоп-битов для RS-485
const STOP_BITS_OPTIONS: (&str, &str) = ("1 стоп-бит", "2 стоп-бита");

/// Структура для хранения настроек подключения
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConnectionSettings {
    port: String,
    device_address: u8,
    baud_rate: u32,
    parity: String,
    stop_bits: u8,
}

/// Структура для метаданных
#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    last_updated: String,
    version: String,
    description: String,
}

/// Структура для описания регистра
#[derive(Serialize, Deserialize, Debug, Clone)]
struct RegisterConfig {
    name: String,
    description: String,
    address: u16,
    var_type: String,
    modbus_type: String,
    enabled: bool,
}

/// Структура для хранения всех регистров
#[derive(Serialize, Deserialize, Debug)]
struct RegistersConfig {
    registers: Vec<RegisterConfig>,
    metadata: Metadata,
}

/// Основная структура конфигурации
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    connection: ConnectionSettings,
    metadata: Metadata,
}

/// Включение поддержки цветного вывода в Windows
#[cfg(windows)]
fn enable_ansi_support() {
    unsafe {
        let stdout = GetStdHandle(STD_OUTPUT_HANDLE);
        let stderr = GetStdHandle(STD_ERROR_HANDLE);

        let mut mode: u32 = 0;
        if GetConsoleMode(stdout, &mut mode) != 0 {
            SetConsoleMode(stdout, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }

        let mut mode: u32 = 0;
        if GetConsoleMode(stderr, &mut mode) != 0 {
            SetConsoleMode(stderr, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}

/// Заглушка для не-Windows систем
#[cfg(not(windows))]
fn enable_ansi_support() {
    // На Unix-системах цветной вывод работает по умолчанию
}

/// scan_available_ports function moved to scan_available_ports module

/// Функция обработки отсутствия портов
fn handle_no_ports() -> io::Result<bool> {
    println!("{}", "Доступные COM-порты не найдены!".red());
    println!("\n{}", "Выберите действие:".yellow());
    println!("  {} - выйти", "0".red());
    println!("  {} - повторить поиск", "1".green());

    loop {
        print!("\nВаш выбор (0-1): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(0) => {
                println!("{}", "Выход из программы...".yellow());
                return Ok(false); // false = выйти
            }
            Ok(1) => {
                println!("{}", "Повторяем поиск портов...".cyan());
                return Ok(true); // true = повторить поиск
            }
            _ => {
                println!(
                    "{}",
                    "Неверный выбор! Введите 0 для выхода или 1 для повторного поиска.".red()
                );
            }
        }
    }
}

/// Функция выбора COM-порта пользователем
fn select_com_port(available_ports: &[u8; 10], ports_count: usize) -> io::Result<Option<String>> {
    if ports_count == 0 {
        return Ok(None);
    }

    println!("\n{}", "Выберите COM-порт для подключения:".cyan());

    // Показываем список доступных портов
    for i in 0..ports_count {
        println!("  {}. COM{}", i + 1, available_ports[i]);
    }

    loop {
        print!("\nВведите номер порта (1-{}): ", ports_count);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(choice) if choice >= 1 && choice <= ports_count => {
                let selected_port = format!("COM{}", available_ports[choice - 1]);
                println!("{}", format!("Выбран порт: {}", selected_port).green());
                return Ok(Some(selected_port));
            }
            _ => {
                println!(
                    "{}",
                    format!("Неверный выбор! Введите число от 1 до {}", ports_count).red()
                );
            }
        }
    }
}

/// Функция выбора адреса устройства
fn select_device_address() -> io::Result<u8> {
    println!("\n{}", "Выбор адреса устройства Modbus".cyan());
    println!("Допустимый диапазон: 1-240");

    loop {
        print!("\nВведите адрес устройства (1-240): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(address) if address >= 1 && address <= 240 => {
                println!(
                    "{}",
                    format!("Выбран адрес устройства: {}", address).green()
                );
                return Ok(address);
            }
            Ok(address) => {
                println!(
                    "{}",
                    format!(
                        "Недопустимый адрес: {}! Введите значение от 1 до 240.",
                        address
                    )
                    .red()
                );
            }
            Err(_) => {
                println!("{}", "Неверный формат! Введите число от 1 до 240.".red());
            }
        }
    }
}

/// Функция выбора скорости передачи данных
fn select_baud_rate() -> io::Result<u32> {
    println!("\n{}", "Выбор скорости передачи данных RS-485".cyan());
    println!("Доступные скорости:");

    // Показываем список доступных скоростей в обратном порядке (от большей к меньшей)
    println!("  1. {} бод", AVAILABLE_BAUD_RATES.6);
    println!("  2. {} бод", AVAILABLE_BAUD_RATES.5);
    println!("  3. {} бод", AVAILABLE_BAUD_RATES.4);
    println!("  4. {} бод", AVAILABLE_BAUD_RATES.3);
    println!("  5. {} бод", AVAILABLE_BAUD_RATES.2);
    println!("  6. {} бод", AVAILABLE_BAUD_RATES.1);
    println!("  7. {} бод", AVAILABLE_BAUD_RATES.0);

    loop {
        print!("\nВведите номер скорости (1-7): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(choice) if choice >= 1 && choice <= 7 => {
                let selected_baud = match choice {
                    1 => AVAILABLE_BAUD_RATES.6,
                    2 => AVAILABLE_BAUD_RATES.5,
                    3 => AVAILABLE_BAUD_RATES.4,
                    4 => AVAILABLE_BAUD_RATES.3,
                    5 => AVAILABLE_BAUD_RATES.2,
                    6 => AVAILABLE_BAUD_RATES.1,
                    7 => AVAILABLE_BAUD_RATES.0,
                    _ => unreachable!(), // Этого никогда не произойдет из-за проверки выше
                };
                println!(
                    "{}",
                    format!("Выбрана скорость: {} бод", selected_baud).green()
                );
                return Ok(selected_baud);
            }
            Ok(choice) => {
                println!(
                    "{}",
                    format!("Недопустимый выбор: {}! Введите число от 1 до 7.", choice).red()
                );
            }
            Err(_) => {
                println!("{}", "Неверный формат! Введите число от 1 до 7.".red());
            }
        }
    }
}

/// Функция выбора четности
fn select_parity() -> io::Result<tokio_serial::Parity> {
    println!("\n{}", "Выбор четности для RS-485".cyan());
    println!("Доступные варианты четности:");

    // Показываем список доступных вариантов четности
    println!("  1. {} - без контроля четности", PARITY_OPTIONS.0);
    println!("  2. {} - четная четность", PARITY_OPTIONS.1);
    println!("  3. {} - нечетная четность", PARITY_OPTIONS.2);

    loop {
        print!("\nВведите номер четности (1-3): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(choice) if choice >= 1 && choice <= 3 => {
                let (selected_parity, parity_name) = match choice {
                    1 => (tokio_serial::Parity::None, PARITY_OPTIONS.0),
                    2 => (tokio_serial::Parity::Even, PARITY_OPTIONS.1),
                    3 => (tokio_serial::Parity::Odd, PARITY_OPTIONS.2),
                    _ => unreachable!(), // Этого никогда не произойдет из-за проверки выше
                };
                println!("{}", format!("Выбрана четность: {}", parity_name).green());
                return Ok(selected_parity);
            }
            Ok(choice) => {
                println!(
                    "{}",
                    format!("Недопустимый выбор: {}! Введите число от 1 до 3.", choice).red()
                );
            }
            Err(_) => {
                println!("{}", "Неверный формат! Введите число от 1 до 3.".red());
            }
        }
    }
}

/// Функция выбора количества стоп-битов
fn select_stop_bits() -> io::Result<tokio_serial::StopBits> {
    println!("\n{}", "Выбор количества стоп-битов для RS-485".cyan());
    println!("Доступные варианты:");

    // Показываем список доступных вариантов стоп-битов
    println!("  1. {} (стандартная настройка)", STOP_BITS_OPTIONS.0);
    println!("  2. {} (повышенная надежность)", STOP_BITS_OPTIONS.1);

    loop {
        print!("\nВведите номер стоп-битов (1-2): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(choice) if choice >= 1 && choice <= 2 => {
                let (selected_stop_bits, stop_bits_name) = match choice {
                    1 => (tokio_serial::StopBits::One, STOP_BITS_OPTIONS.0),
                    2 => (tokio_serial::StopBits::Two, STOP_BITS_OPTIONS.1),
                    _ => unreachable!(), // Этого никогда не произойдет из-за проверки выше
                };
                println!("{}", format!("Выбрано: {}", stop_bits_name).green());
                return Ok(selected_stop_bits);
            }
            Ok(choice) => {
                println!(
                    "{}",
                    format!("Недопустимый выбор: {}! Введите число от 1 до 2.", choice).red()
                );
            }
            Err(_) => {
                println!("{}", "Неверный формат! Введите число от 1 до 2.".red());
            }
        }
    }
}

/// Функция получения пути к файлу настроек
fn get_settings_path() -> String {
    // В режиме разработки (cargo run) - в корне проекта
    // В режиме release (exe файл) - рядом с exe файлом
    if cfg!(debug_assertions) {
        // Режим разработки - файл в корне проекта
        "connect_settings.json".to_string()
    } else {
        // Режим release - файл рядом с exe
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
fn get_registers_path() -> String {
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

/// Функция загрузки настроек из JSON файла
fn load_settings() -> io::Result<Config> {
    let settings_path = get_settings_path();
    let file_content = fs::read_to_string(&settings_path)?;
    let config: Config = serde_json::from_str(&file_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(config)
}

/// Функция загрузки конфигурации регистров из CSV файла
fn load_registers() -> io::Result<RegistersConfig> {
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
fn load_registers_or_warn() -> Option<RegistersConfig> {
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

/// Функция сохранения настроек в JSON файл
fn save_settings(connection: ConnectionSettings) -> io::Result<()> {
    let metadata = Metadata {
        last_updated: chrono::Utc::now().to_rfc3339(),
        version: "1.0".to_string(),
        description: "Настройки подключения для Modbus RTU через RS-485".to_string(),
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

/// Функция отображения настроек связи
fn show_connection_settings() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Текущие настройки связи ===".cyan().bold());

    match load_settings() {
        Ok(config) => {
            let conn = &config.connection;
            println!("\n{}", "Параметры подключения:".yellow());
            println!("  {} {}", "COM-порт:".green(), conn.port.bright_white());
            println!(
                "  {} {}",
                "Адрес устройства:".green(),
                conn.device_address.to_string().bright_white()
            );
            println!(
                "  {} {} бод",
                "Скорость:".green(),
                conn.baud_rate.to_string().bright_white()
            );
            println!("  {} {}", "Четность:".green(), conn.parity.bright_white());

            let stop_bits_text = match conn.stop_bits {
                1 => "1 стоп-бит",
                2 => "2 стоп-бита",
                _ => "неизвестно",
            };
            println!(
                "  {} {}",
                "Стоп-биты:".green(),
                stop_bits_text.bright_white()
            );

            println!("\n{}", "Информация о файле:".yellow());
            println!(
                "  {} {}",
                "Версия:".blue(),
                config.metadata.version.bright_white()
            );
            println!(
                "  {} {}",
                "Обновлен:".blue(),
                config.metadata.last_updated.bright_white()
            );
        }
        Err(e) => {
            eprintln!("{}", format!("Ошибка загрузки настроек: {}", e).red());
            println!("{}", "Будут использованы настройки по умолчанию.".yellow());
        }
    }

    Ok(())
}

/// Функция очистки экрана консоли
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Функция ожидания нажатия Enter для продолжения
fn wait_for_continue() -> io::Result<()> {
    println!("\n{}", "Нажмите Enter для продолжения...".bright_black());
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

/// Функция для обработки данных регистра в зависимости от типа
fn process_register_data(data: &[u16], register: &RegisterConfig) -> String {
    match register.var_type.as_str() {
        "bool" => {
            if data.len() >= 1 {
                if data[0] != 0 {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "u16" => {
            if data.len() >= 1 {
                format!("{}", data[0])
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "i16" => {
            if data.len() >= 1 {
                let value = data[0] as i16;
                format!("{}", value)
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "u32" | "i32" => {
            if data.len() >= 2 {
                // Объединяем два 16-битных регистра в одно 32-битное значение
                let high_word = data[1] as u32;
                let low_word = data[0] as u32;
                let combined = (high_word << 16) | low_word;

                if register.var_type == "i32" {
                    let value = combined as i32;
                    format!("{}", value)
                } else {
                    format!("{}", combined)
                }
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "float" => {
            if data.len() >= 2 {
                // Объединяем два 16-битных регистра в float (IEEE 754)
                let high_word = data[1] as u32;
                let low_word = data[0] as u32;
                let combined = (high_word << 16) | low_word;
                let value = f32::from_bits(combined);
                format!("{:.3}", value)
            } else {
                "Недостаточно данных".to_string()
            }
        }
        _ => {
            // Неизвестный тип - показываем как массив u16
            let values: Vec<String> = data.iter().map(|&x| format!("{}", x)).collect();
            format!("[{}]", values.join(", "))
        }
    }
}

/// Вычисляет количество 16-битных регистров для чтения по типу переменной
fn compute_quantity(var_type: &str) -> u16 {
    match var_type {
        "bool" | "u16" | "i16" => 1,
        "u32" | "i32" | "float" => 2,
        _ => 1,
    }
}

/// Единообразная обработка результата чтения регистра с таймаутом
fn print_register_result<T, E, F>(
    register: &RegisterConfig,
    result: Result<Result<T, E>, tokio::time::error::Elapsed>,
    mut format_ok: F,
    all_success: &mut bool,
) where
    E: std::fmt::Debug,
    F: FnMut(T) -> String,
{
    match result {
        Ok(Ok(data)) => {
            let processed_value = format_ok(data);
            print!("{}: {} | ", register.name.cyan(), processed_value.green());
        }
        Ok(Err(e)) => {
            print!(
                "{}: {} | ",
                register.name.cyan(),
                format!("Ошибка: {:?}", e).red()
            );
            *all_success = false;
        }
        Err(_) => {
            print!("{}: {} | ", register.name.cyan(), "Таймаут".red());
            *all_success = false;
        }
    }
}

/// Форматирование значения для булевых регистров (coil/discrete_input)
fn format_bool_value(data: Vec<bool>) -> String {
    if data.first().copied().unwrap_or(false) {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

/// Функция изменения настроек связи
fn change_connection_settings() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Изменение настроек связи ===".cyan().bold());
    println!();

    // Сканирование доступных портов
    let mut available_ports: [u8; 10] = [0; 10];
    let ports_count = scan_available_ports(&mut available_ports);

    // Выбор COM-порта
    let port = loop {
        match select_com_port(&available_ports, ports_count)? {
            Some(port) => break port,
            None => {
                if !handle_no_ports()? {
                    return Ok(()); // Пользователь выбрал выход
                }
                println!(); // Пустая строка для разделения
                let ports_count = scan_available_ports(&mut available_ports);
                if ports_count == 0 {
                    continue;
                }
            }
        }
    };

    // Выбор адреса устройства
    let device_address = select_device_address()?;

    // Выбор скорости передачи данных
    let baud_rate = select_baud_rate()?;

    // Выбор четности
    let parity_enum = select_parity()?;
    let parity = match parity_enum {
        tokio_serial::Parity::None => "None".to_string(),
        tokio_serial::Parity::Even => "Even".to_string(),
        tokio_serial::Parity::Odd => "Odd".to_string(),
    };

    // Выбор количества стоп-битов
    let stop_bits_enum = select_stop_bits()?;
    let stop_bits = match stop_bits_enum {
        tokio_serial::StopBits::One => 1,
        tokio_serial::StopBits::Two => 2,
    };

    // Создание структуры настроек
    let connection_settings = ConnectionSettings {
        port,
        device_address,
        baud_rate,
        parity,
        stop_bits,
    };

    // Сохранение настроек в файл
    match save_settings(connection_settings) {
        Ok(()) => {
            println!("\n{}", "Настройки успешно сохранены!".green().bold());
        }
        Err(e) => {
            eprintln!("{}", format!("Ошибка сохранения настроек: {}", e).red());
        }
    }

    Ok(())
}

/// Функция запуска опроса с использованием сохраненных настроек
async fn start_polling() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Запуск опроса устройства ===".cyan().bold());

    // Загрузка настроек подключения
    let config = match load_settings() {
        Ok(config) => {
            println!("{}", "Настройки подключения успешно загружены".green());
            config
        }
        Err(e) => {
            eprintln!(
                "{}",
                format!("Ошибка загрузки настроек подключения: {}", e).red()
            );
            println!(
                "{}",
                "Убедитесь, что настройки сохранены (пункт 2 в главном меню)".yellow()
            );
            return Err(e);
        }
    };

    // Загрузка конфигурации регистров
    let registers_config = match load_registers() {
        Ok(registers_config) => {
            println!("{}", "Конфигурация регистров успешно загружена".green());
            registers_config
        }
        Err(e) => {
            eprintln!(
                "{}",
                format!("Ошибка загрузки конфигурации регистров: {}", e).red()
            );
            println!(
                "{}",
                "Убедитесь, что файл tags.csv существует и корректен".yellow()
            );
            return Err(e);
        }
    };

    let conn = &config.connection;
    let enabled_registers: Vec<&RegisterConfig> = registers_config
        .registers
        .iter()
        .filter(|reg| reg.enabled)
        .collect();

    if enabled_registers.is_empty() {
        println!("{}", "Нет активных регистров для опроса!".red());
        println!(
            "{}",
            "Проверьте файл tags.csv и убедитесь, что есть регистры с enabled: true".yellow()
        );
        return Ok(());
    }

    println!("Используемые настройки подключения:");
    println!("  COM-порт: {}", conn.port.bright_white());
    println!(
        "  Адрес устройства: {}",
        conn.device_address.to_string().bright_white()
    );
    println!(
        "  Скорость: {} бод",
        conn.baud_rate.to_string().bright_white()
    );
    println!("  Четность: {}", conn.parity.bright_white());

    let stop_bits_text = match conn.stop_bits {
        1 => "1 стоп-бит",
        2 => "2 стоп-бита",
        _ => "неизвестно",
    };
    println!("  Стоп-биты: {}", stop_bits_text.bright_white());

    println!("\nАктивные регистры для опроса:");
    for register in &enabled_registers {
        let qty = compute_quantity(&register.var_type);
        println!(
            "  {} (адрес: {}, тип: {}, количество: {})",
            register.name.cyan(),
            register.address,
            register.var_type.yellow(),
            qty
        );
    }
    println!();

    // Преобразование настроек для tokio_serial
    let parity = match conn.parity.as_str() {
        "None" => tokio_serial::Parity::None,
        "Even" => tokio_serial::Parity::Even,
        "Odd" => tokio_serial::Parity::Odd,
        _ => tokio_serial::Parity::None,
    };

    let stop_bits = match conn.stop_bits {
        1 => tokio_serial::StopBits::One,
        2 => tokio_serial::StopBits::Two,
        _ => tokio_serial::StopBits::One,
    };

    // Настройка параметров последовательного порта
    let builder = tokio_serial::new(&conn.port, conn.baud_rate)
        .data_bits(tokio_serial::DataBits::Eight)
        .parity(parity)
        .stop_bits(stop_bits);

    // Открытие последовательного порта
    let port = match SerialStream::open(&builder) {
        Ok(port) => {
            println!(
                "{}",
                format!("Последовательный порт {} успешно открыт", conn.port).green()
            );
            port
        }
        Err(e) => {
            eprintln!(
                "{}",
                format!(
                    "Ошибка открытия последовательного порта {}: {:?}",
                    conn.port, e
                )
                .red()
            );
            return Err(e.into());
        }
    };

    // Создание контекста Modbus RTU
    let mut ctx = match rtu::connect(port).await {
        Ok(ctx) => {
            println!("{}", "Modbus RTU контекст успешно создан".green());
            ctx
        }
        Err(e) => {
            eprintln!(
                "{}",
                format!("Ошибка создания Modbus RTU контекста: {:?}", e).red()
            );
            return Err(e.into());
        }
    };

    // Установка адреса устройства
    let slave_addr = Slave(conn.device_address);
    ctx.set_slave(slave_addr);

    // Циклический опрос устройства каждую секунду
    println!(
        "{}",
        "Начинается циклический опрос устройства (каждую секунду)...".cyan()
    );
    println!("{}", "Нажмите Ctrl+C для остановки опроса".yellow());
    println!();

    let timeout_duration = Duration::from_millis(1000);
    let mut error_count = 0;

    loop {
        // Показываем только время
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        print!("{} ", timestamp.to_string().bright_black());

        let mut all_success = true;

        // Опрашиваем каждый активный регистр
        for register in &enabled_registers {
            match register.modbus_type.as_str() {
                "input_register" => {
                    let qty = compute_quantity(&register.var_type);
                    let result = tokio::time::timeout(
                        timeout_duration,
                        ctx.read_input_registers(register.address, qty),
                    )
                    .await;

                    print_register_result(
                        register,
                        result,
                        |data: Vec<u16>| process_register_data(&data, register),
                        &mut all_success,
                    );
                }
                "holding_register" => {
                    let qty = compute_quantity(&register.var_type);
                    let result = tokio::time::timeout(
                        timeout_duration,
                        ctx.read_holding_registers(register.address, qty),
                    )
                    .await;

                    print_register_result(
                        register,
                        result,
                        |data: Vec<u16>| process_register_data(&data, register),
                        &mut all_success,
                    );
                }
                "coil" => {
                    let qty: u16 = 1;
                    let result = tokio::time::timeout(
                        timeout_duration,
                        ctx.read_coils(register.address, qty),
                    )
                    .await;

                    print_register_result(register, result, format_bool_value, &mut all_success);
                }
                "discrete_input" => {
                    let qty: u16 = 1;
                    let result = tokio::time::timeout(
                        timeout_duration,
                        ctx.read_discrete_inputs(register.address, qty),
                    )
                    .await;

                    print_register_result(register, result, format_bool_value, &mut all_success);
                }
                _ => {
                    println!("Неизвестный тип регистра: {}", register.modbus_type.red());
                    continue;
                }
            }
        }

        // Обновляем счетчик ошибок
        if all_success {
            error_count = 0;
        } else {
            error_count += 1;
        }

        if !all_success {
            print!("{}", format!("(errors: {})", error_count).yellow());
        }

        println!(); // Переход на новую строку

        // Ожидание 1 секунды перед следующим опросом
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Функция отображения всех регистров и их настроек
fn show_registers() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Конфигурация регистров ===".cyan().bold());

    match load_registers() {
        Ok(registers_config) => {
            println!("{}", "Регистры успешно загружены из файла".green());

            // Показываем метаданные
            println!("\n{}", "Информация о конфигурации:".yellow());
            println!(
                "  {} {}",
                "Версия:".blue(),
                registers_config.metadata.version.bright_white()
            );
            println!(
                "  {} {}",
                "Описание:".blue(),
                registers_config.metadata.description.bright_white()
            );
            println!(
                "  {} {}",
                "Обновлено:".blue(),
                registers_config.metadata.last_updated.bright_white()
            );

            // Показываем статистику
            let total_count = registers_config.registers.len();
            let enabled_count = registers_config
                .registers
                .iter()
                .filter(|reg| reg.enabled)
                .count();
            let disabled_count = total_count - enabled_count;

            println!("\n{}", "Статистика регистров:".yellow());
            println!(
                "  {} {}",
                "Всего регистров:".blue(),
                total_count.to_string().bright_white()
            );
            println!(
                "  {} {}",
                "Активных:".green(),
                enabled_count.to_string().bright_white()
            );
            println!(
                "  {} {}",
                "Отключенных:".red(),
                disabled_count.to_string().bright_white()
            );

            if registers_config.registers.is_empty() {
                println!("\n{}", "Регистры не найдены!".red());
            } else {
                println!("\n{}", "Список регистров:".yellow());
                println!("{}", "─".repeat(120));
                println!(
                    "{:<3} {:<20} {:<40} {:<8} {:<10} {:<20} {:<10}",
                    "#", "Имя", "Описание", "Адрес", "Тип", "Modbus тип", "Статус"
                );
                println!("{}", "─".repeat(120));

                for (index, register) in registers_config.registers.iter().enumerate() {
                    let status = if register.enabled {
                        "Активен".green()
                    } else {
                        "Отключен".red()
                    };

                    let name = if register.name.chars().count() > 19 {
                        let truncated: String = register.name.chars().take(16).collect();
                        format!("{}...", truncated)
                    } else {
                        register.name.clone()
                    };

                    let description = if register.description.chars().count() > 39 {
                        let truncated: String = register.description.chars().take(36).collect();
                        format!("{}...", truncated)
                    } else {
                        register.description.clone()
                    };

                    println!(
                        "{:<3} {:<20} {:<40} {:<8} {:<10} {:<20} {}",
                        (index + 1).to_string().bright_black(),
                        name.cyan(),
                        description,
                        register.address.to_string().bright_white(),
                        register.var_type.yellow(),
                        register.modbus_type.blue(),
                        status
                    );
                }
                println!("{}", "─".repeat(120));
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Ошибка загрузки регистров: {}", e).red());
            println!(
                "{}",
                "Убедитесь, что файл tags.csv существует и корректен".yellow()
            );
        }
    }

    Ok(())
}

/// Сохранение регистров обратно в CSV (tags.csv)
fn save_registers_to_csv(registers: &[RegisterConfig]) -> io::Result<()> {
    let path = get_registers_path();
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path(&path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Заголовок
    writer
        .write_record([
            "name",
            "description",
            "address",
            "var_type",
            "modbus_type",
            "enabled",
        ])
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    writer
        .flush()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}

/// Удаление регистра по порядковому номеру (интерактивно)
fn delete_register() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Удаление регистра ===".cyan().bold());

    let mut cfg = match load_registers_or_warn() {
        Some(c) => c,
        None => {
            return Ok(());
        }
    };

    if cfg.registers.is_empty() {
        println!("{}", "Список регистров пуст — удалять нечего".yellow());
        wait_for_continue()?;
        return Ok(());
    }

    println!("\n{}", "Список регистров:".yellow());
    for (idx, reg) in cfg.registers.iter().enumerate() {
        println!(
            "  {:<3} {:<20} (адрес: {:<5} тип: {:<6} modbus: {:<16})",
            (idx + 1).to_string().bright_black(),
            reg.name.cyan(),
            reg.address,
            reg.var_type.yellow(),
            reg.modbus_type.blue()
        );
    }

    print!(
        "\nВведите номер регистра для удаления (1-{}), либо 0 для отмены: ",
        cfg.registers.len()
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    let Ok(num) = trimmed.parse::<usize>() else {
        println!("{}", "Неверный ввод. Ожидалось число.".yellow());
        wait_for_continue()?;
        return Ok(());
    };

    if num == 0 {
        println!("{}", "Удаление отменено".bright_black());
        wait_for_continue()?;
        return Ok(());
    }

    if num < 1 || num > cfg.registers.len() {
        println!(
            "{}",
            format!("Номер вне диапазона (1-{})", cfg.registers.len()).yellow()
        );
        wait_for_continue()?;
        return Ok(());
    }

    let removed = cfg.registers.remove(num - 1);
    save_registers_to_csv(&cfg.registers)?;
    println!(
        "{}",
        format!(
            "Регистр '{}' (адрес {}) удалён",
            removed.name, removed.address
        )
        .green()
    );
    wait_for_continue()?;
    Ok(())
}

/// Функция отображения меню регистров
fn show_registers_menu() -> io::Result<u8> {
    clear_screen();
    println!("{}", "=== Управление регистрами ===".cyan().bold());
    println!("\n{}", "Выберите действие:".yellow());
    println!("  {} - Показать регистры", "1".green());
    println!("  {} - Удалить регистр", "2".red());
    println!("  {} - Добавить регистр", "3".blue());
    println!("  {} - Изменить регистр", "4".magenta());
    println!("  {} - Отсортировать по адресу", "5".cyan());
    println!("  {} - Назад в главное меню", "9".bright_black());

    print!("\nВаш выбор (1-5, 9): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().parse::<u8>() {
        Ok(1) | Ok(2) | Ok(3) | Ok(4) | Ok(5) | Ok(9) => Ok(input.trim().parse().unwrap()),
        _ => {
            println!(
                "{}",
                "Неверный выбор! Возвращаемся в главное меню.".yellow()
            );
            Ok(9)
        }
    }
}

/// Функция отображения главного меню
fn show_main_menu() -> io::Result<u8> {
    clear_screen();
    println!("{}", "=== Modbus RTU Client ===".cyan().bold());
    println!("\n{}", "Выберите действие:".yellow());
    println!("  {} - Показать настройки связи", "1".green());
    println!("  {} - Изменить настройки связи", "2".blue());
    println!("  {} - Начать опрос", "3".magenta());
    println!("  {} - Регистры", "4".bright_blue());
    println!("  {} - Выйти", "9".red());

    print!("\nВаш выбор (1-4, 9): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().parse::<u8>() {
        Ok(1) | Ok(2) | Ok(3) | Ok(4) | Ok(9) => Ok(input.trim().parse().unwrap()),
        _ => {
            println!(
                "{}",
                "Неверный выбор! Используется пункт 3 по умолчанию.".yellow()
            );
            Ok(3)
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Включение поддержки цветного вывода в Windows
    enable_ansi_support();

    // Главный цикл программы
    loop {
        let choice = show_main_menu()?;

        match choice {
            1 => {
                // Показать настройки связи
                show_connection_settings()?;
                wait_for_continue()?;
                println!(); // Пустая строка для разделения
                continue; // Возвращаемся к главному меню
            }
            2 => {
                // Изменить настройки связи
                change_connection_settings()?;
                wait_for_continue()?;
                continue; // Возвращаемся к главному меню
            }
            3 => {
                // Начать опрос с использованием сохраненных настроек
                match start_polling().await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        eprintln!("{}", format!("Ошибка при опросе: {}", e).red());
                        wait_for_continue()?;
                        continue; // Возвращаемся к главному меню
                    }
                }
            }
            4 => {
                // Меню управления регистрами
                loop {
                    let registers_choice = show_registers_menu()?;
                    match registers_choice {
                        1 => {
                            // Показать регистры
                            show_registers()?;
                            wait_for_continue()?;
                        }
                        2 => {
                            // Удалить регистр
                            delete_register()?;
                        }
                        3 => {
                            // Добавить регистр
                            add_register()?;
                        }
                        4 => {
                            // Изменить регистр (пока не реализовано)
                            println!(
                                "{}",
                                "Функция 'Изменить регистр' пока не реализована".yellow()
                            );
                            wait_for_continue()?;
                        }
                        5 => {
                            // Отсортировать по адресу
                            match sort_registers::sort_registers_by_address() {
                                Ok(()) => {
                                    println!("{}", "Сортировка завершена".green());
                                }
                                Err(e) => {
                                    eprintln!("{}", format!("Ошибка сортировки: {}", e).red());
                                }
                            }
                            wait_for_continue()?;
                        }
                        9 => {
                            // Назад в главное меню
                            break;
                        }
                        _ => unreachable!(), // Этого не произойдет из-за проверки в show_registers_menu
                    }
                }
                continue; // Возвращаемся к главному меню
            }
            9 => {
                println!("{}", "Завершение программы...".yellow());
                return Ok(()); // Завершаем программу
            }
            _ => unreachable!(), // Этого не произойдет из-за проверки в show_main_menu
        }
    }
}
