use colored::*;
use std::io::{self, Write};

use crate::config::model::RegisterConfig;
use crate::persistence::registers_csv::{load_registers, load_registers_or_warn, save_registers_to_csv};
use crate::terminal::{clear_screen, wait_for_continue};

/// Функция отображения всех регистров и их настроек
pub fn show_registers() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Конфигурация регистров ===".cyan().bold());

    match load_registers() {
        Ok(registers_config) => {
            println!("{}", "Регистры успешно загружены из файла".green());
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

/// Удаление регистра по порядковому номеру (интерактивно)
pub fn delete_register() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Удаление регистра ===".cyan().bold());

    let mut cfg = match load_registers_or_warn() {
        Some(c) => c,
        None => return Ok(()),
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
        format!("Регистр '{}' (адрес {}) удалён", removed.name, removed.address).green()
    );
    wait_for_continue()?;
    Ok(())
}

/// Удаление всех регистров (с подтверждением)
pub fn delete_all_registers() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Удаление всех регистров ===".red().bold());

    let cfg = match load_registers_or_warn() {
        Some(c) => c,
        None => return Ok(()),
    };

    if cfg.registers.is_empty() {
        println!("{}", "Список регистров уже пуст".yellow());
        wait_for_continue()?;
        return Ok(());
    }

    let total_count = cfg.registers.len();
    println!("\n{}", format!("Всего регистров в списке: {}", total_count).yellow());
    println!("{}", "ВНИМАНИЕ! Это действие удалит ВСЕ регистры из файла tags.csv!".red().bold());
    println!("{}", "Это действие необратимо!".red());

    print!(
        "\n{} ",
        "Вы уверены? Введите 'ДА' для подтверждения или любой другой текст для отмены:".yellow()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let confirmation = input.trim();

    if confirmation == "ДА" || confirmation == "да" || confirmation == "YES" || confirmation == "yes" {
        let empty_registers: Vec<RegisterConfig> = Vec::new();
        save_registers_to_csv(&empty_registers)?;
        println!(
            "\n{}",
            format!("Все регистры ({} шт.) успешно удалены", total_count).green().bold()
        );
        println!("{}", "Файл tags.csv теперь содержит только заголовок".bright_black());
    } else {
        println!("\n{}", "Удаление отменено".bright_black());
    }

    wait_for_continue()?;
    Ok(())
}

/// Функция отображения меню регистров
pub fn show_registers_menu() -> io::Result<u8> {
    clear_screen();
    println!("{}", "=== Управление регистрами ===".cyan().bold());
    println!("\n{}", "Выберите действие:".yellow());
    println!("  {} - Показать регистры", "1".green());
    println!("  {} - Удалить регистр", "2".red());
    println!("  {} - Добавить регистр", "3".blue());
    println!("  {} - Изменить регистр", "4".magenta());
    println!("  {} - Отсортировать по адресу", "5".cyan());
    println!("  {} - Удалить все регистры", "6".red().bold());
    println!("  {} - Назад в главное меню", "9".bright_black());

    print!("\nВаш выбор (1-6, 9): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().parse::<u8>() {
        Ok(1) | Ok(2) | Ok(3) | Ok(4) | Ok(5) | Ok(6) | Ok(9) => Ok(input.trim().parse().unwrap()),
        _ => {
            println!(
                "{}",
                "Неверный выбор! Возвращаемся в главное меню.".yellow()
            );
            Ok(9)
        }
    }
}
