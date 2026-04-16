use colored::*;
use std::io::{self, Write};

use crate::config::constants::{AVAILABLE_BAUD_RATES, PARITY_OPTIONS, STOP_BITS_OPTIONS};

/// Функция обработки отсутствия портов
pub fn handle_no_ports() -> io::Result<bool> {
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
                return Ok(false);
            }
            Ok(1) => {
                println!("{}", "Повторяем поиск портов...".cyan());
                return Ok(true);
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
pub fn select_com_port(available_ports: &[u8; 10], ports_count: usize) -> io::Result<Option<String>> {
    if ports_count == 0 {
        return Ok(None);
    }

    println!("\n{}", "Выберите COM-порт для подключения:".cyan());
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
pub fn select_device_address() -> io::Result<u8> {
    println!("\n{}", "Выбор адреса устройства Modbus".cyan());
    println!("Допустимый диапазон: 1-240");

    loop {
        print!("\nВведите адрес устройства (1-240): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(address) if (1..=240).contains(&address) => {
                println!("{}", format!("Выбран адрес устройства: {}", address).green());
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

pub fn select_tcp_host(current_host: &str) -> io::Result<String> {
    println!("\n{}", "Настройка TCP хоста".cyan());
    println!(
        "{}",
        format!("Текущее значение: {}", current_host).bright_black()
    );

    print!("Введите IP/хост (Enter = оставить текущее): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(current_host.to_string());
    }

    Ok(trimmed.to_string())
}

pub fn select_tcp_port(current_port: u16) -> io::Result<u16> {
    println!("\n{}", "Настройка TCP порта".cyan());
    println!(
        "{}",
        format!("Текущее значение: {}", current_port).bright_black()
    );

    loop {
        print!("Введите TCP порт (1-65535, Enter = оставить текущий): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(current_port);
        }

        match trimmed.parse::<u16>() {
            Ok(port) if port >= 1 => return Ok(port),
            _ => println!("{}", "Неверный порт! Введите число от 1 до 65535.".red()),
        }
    }
}

/// Функция выбора скорости передачи данных
pub fn select_baud_rate() -> io::Result<u32> {
    println!("\n{}", "Выбор скорости передачи данных RS-485".cyan());
    println!("Доступные скорости:");
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
            Ok(choice) if (1..=7).contains(&choice) => {
                let selected_baud = match choice {
                    1 => AVAILABLE_BAUD_RATES.6,
                    2 => AVAILABLE_BAUD_RATES.5,
                    3 => AVAILABLE_BAUD_RATES.4,
                    4 => AVAILABLE_BAUD_RATES.3,
                    5 => AVAILABLE_BAUD_RATES.2,
                    6 => AVAILABLE_BAUD_RATES.1,
                    7 => AVAILABLE_BAUD_RATES.0,
                    _ => unreachable!(),
                };
                println!("{}", format!("Выбрана скорость: {} бод", selected_baud).green());
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
pub fn select_parity() -> io::Result<tokio_serial::Parity> {
    println!("\n{}", "Выбор четности для RS-485".cyan());
    println!("Доступные варианты четности:");
    println!("  1. {} - без контроля четности", PARITY_OPTIONS.0);
    println!("  2. {} - четная четность", PARITY_OPTIONS.1);
    println!("  3. {} - нечетная четность", PARITY_OPTIONS.2);

    loop {
        print!("\nВведите номер четности (1-3): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(choice) if (1..=3).contains(&choice) => {
                let (selected_parity, parity_name) = match choice {
                    1 => (tokio_serial::Parity::None, PARITY_OPTIONS.0),
                    2 => (tokio_serial::Parity::Even, PARITY_OPTIONS.1),
                    3 => (tokio_serial::Parity::Odd, PARITY_OPTIONS.2),
                    _ => unreachable!(),
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
pub fn select_stop_bits() -> io::Result<tokio_serial::StopBits> {
    println!("\n{}", "Выбор количества стоп-битов для RS-485".cyan());
    println!("Доступные варианты:");
    println!("  1. {} (стандартная настройка)", STOP_BITS_OPTIONS.0);
    println!("  2. {} (повышенная надежность)", STOP_BITS_OPTIONS.1);

    loop {
        print!("\nВведите номер стоп-битов (1-2): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<u8>() {
            Ok(choice) if (1..=2).contains(&choice) => {
                let (selected_stop_bits, stop_bits_name) = match choice {
                    1 => (tokio_serial::StopBits::One, STOP_BITS_OPTIONS.0),
                    2 => (tokio_serial::StopBits::Two, STOP_BITS_OPTIONS.1),
                    _ => unreachable!(),
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
