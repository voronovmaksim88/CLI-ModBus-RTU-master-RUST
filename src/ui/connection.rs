use colored::*;
use std::io;

use crate::config::constants::{MODE_RTU, MODE_TCP};
use crate::config::model::{ConnectionSettings, is_tcp_mode};
use crate::persistence::settings_json::{create_default_settings, load_settings, save_settings};
use crate::prompts::connection::{
    handle_no_ports, select_baud_rate, select_com_port, select_device_address, select_parity,
    select_stop_bits, select_tcp_host, select_tcp_port,
};
use crate::scan_available_ports::scan_available_ports;
use crate::terminal::clear_screen;

/// Функция отображения настроек связи
pub fn show_connection_settings() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Текущие настройки связи ===".cyan().bold());

    match load_settings() {
        Ok(config) => {
            let conn = &config.connection;
            println!("\n{}", "Параметры подключения:".yellow());
            println!("  {} {}", "Режим:".green(), conn.mode.bright_white());
            println!(
                "  {} {}",
                "Адрес устройства:".green(),
                conn.device_address.to_string().bright_white()
            );
            if is_tcp_mode(&conn.mode) {
                println!("  {} {}", "TCP хост:".green(), conn.tcp_host.bright_white());
                println!(
                    "  {} {}",
                    "TCP порт:".green(),
                    conn.tcp_port.to_string().bright_white()
                );
            } else {
                println!("  {} {}", "COM-порт:".green(), conn.port.bright_white());
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
            }

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

/// Функция изменения настроек связи
pub fn change_connection_settings() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Изменение настроек связи ===".cyan().bold());
    println!();
    let current_settings = match load_settings() {
        Ok(config) => config.connection,
        Err(_) => create_default_settings(),
    };

    let connection_settings = if is_tcp_mode(&current_settings.mode) {
        println!("{}", "Текущий режим: Modbus TCP".yellow());

        let tcp_host = select_tcp_host(&current_settings.tcp_host)?;
        let tcp_port = select_tcp_port(current_settings.tcp_port)?;
        let device_address = select_device_address()?;

        ConnectionSettings {
            port: current_settings.port,
            device_address,
            baud_rate: current_settings.baud_rate,
            parity: current_settings.parity,
            stop_bits: current_settings.stop_bits,
            mode: MODE_TCP.to_string(),
            tcp_host,
            tcp_port,
        }
    } else {
        println!("{}", "Текущий режим: Modbus RTU".yellow());

        let mut available_ports: [u8; 10] = [0; 10];
        let ports_count = scan_available_ports(&mut available_ports);

        let port = loop {
            match select_com_port(&available_ports, ports_count)? {
                Some(port) => break port,
                None => {
                    if !handle_no_ports()? {
                        return Ok(());
                    }
                    println!();
                    let ports_count = scan_available_ports(&mut available_ports);
                    if ports_count == 0 {
                        continue;
                    }
                }
            }
        };

        let device_address = select_device_address()?;
        let baud_rate = select_baud_rate()?;

        let parity_enum = select_parity()?;
        let parity = match parity_enum {
            tokio_serial::Parity::None => "None".to_string(),
            tokio_serial::Parity::Even => "Even".to_string(),
            tokio_serial::Parity::Odd => "Odd".to_string(),
        };

        let stop_bits_enum = select_stop_bits()?;
        let stop_bits = match stop_bits_enum {
            tokio_serial::StopBits::One => 1,
            tokio_serial::StopBits::Two => 2,
        };

        ConnectionSettings {
            port,
            device_address,
            baud_rate,
            parity,
            stop_bits,
            mode: MODE_RTU.to_string(),
            tcp_host: current_settings.tcp_host,
            tcp_port: current_settings.tcp_port,
        }
    };

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

pub fn switch_connection_mode() -> io::Result<()> {
    let mut conn = match load_settings() {
        Ok(config) => config.connection,
        Err(_) => create_default_settings(),
    };

    if is_tcp_mode(&conn.mode) {
        conn.mode = MODE_RTU.to_string();
        println!("{}", "Режим связи переключен на Modbus RTU".green().bold());
    } else {
        conn.mode = MODE_TCP.to_string();
        println!("{}", "Режим связи переключен на Modbus TCP".green().bold());
    }

    save_settings(conn)
}
