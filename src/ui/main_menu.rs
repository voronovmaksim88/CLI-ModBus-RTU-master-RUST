use colored::*;
use std::io::{self, Write};

use crate::config::model::{default_connection_mode, is_tcp_mode};
use crate::persistence::settings_json::load_settings;
use crate::terminal::clear_screen;

/// Функция отображения главного меню
pub fn show_main_menu() -> io::Result<u8> {
    clear_screen();
    println!("{}", "=== Modbus RTU Server ===".cyan().bold());
    let current_mode = match load_settings() {
        Ok(config) => config.connection.mode,
        Err(_) => default_connection_mode(),
    };
    let switch_mode_label = if is_tcp_mode(&current_mode) {
        "Перейти в режим TRU"
    } else {
        "Перейти в режим TCP"
    };

    println!(
        "{} {}",
        "Текущий режим:".yellow(),
        current_mode.bright_white()
    );
    println!("\n{}", "Выберите действие:".yellow());
    println!("  {} - Показать настройки связи", "1".green());
    println!("  {} - Изменить настройки связи", "2".blue());
    println!("  {} - Начать опрос", "3".magenta());
    println!("  {} - Регистры", "4".bright_blue());
    println!("  {} - {}", "5".cyan(), switch_mode_label);
    println!("  {} - Выйти", "9".red());

    print!("\nВаш выбор (1-5, 9): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().parse::<u8>() {
        Ok(1) | Ok(2) | Ok(3) | Ok(4) | Ok(5) | Ok(9) => Ok(input.trim().parse().unwrap()),
        _ => {
            println!(
                "{}",
                "Неверный выбор! Используется пункт 3 по умолчанию.".yellow()
            );
            Ok(3)
        }
    }
}
