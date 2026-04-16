use colored::*;
use std::io;
use std::net::ToSocketAddrs;
use std::time::Duration;
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

use crate::config::model::{RegisterConfig, is_tcp_mode};
use crate::persistence::registers_csv::load_registers;
use crate::persistence::settings_json::load_settings;
use crate::register::value::{compute_quantity, format_bool_value, process_register_data};
use crate::terminal::{clear_screen, wait_for_continue};

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

async fn run_polling_loop(ctx: &mut Context, enabled_registers: &[&RegisterConfig]) -> io::Result<()> {
    println!(
        "{}",
        "Начинается циклический опрос устройства (каждую секунду)...".cyan()
    );
    println!("{}", "Нажмите Ctrl+C для остановки опроса".yellow());
    println!();

    let timeout_duration = Duration::from_millis(1000);
    let mut error_count = 0;

    loop {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        print!("{} ", timestamp.to_string().bright_black());

        let mut all_success = true;

        for register in enabled_registers {
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
                    let result =
                        tokio::time::timeout(timeout_duration, ctx.read_coils(register.address, qty))
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

        if all_success {
            error_count = 0;
        } else {
            error_count += 1;
        }
        if !all_success {
            print!("{}", format!("(errors: {})", error_count).yellow());
        }

        println!();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Функция запуска опроса с использованием сохраненных настроек
pub async fn start_polling() -> io::Result<()> {
    clear_screen();
    println!("{}", "=== Запуск опроса устройства ===".cyan().bold());

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

    if registers_config.registers.is_empty() {
        println!("\n{}", "ОШИБКА: Список регистров пуст!".red().bold());
        println!("{}", "Невозможно начать опрос без регистров.".yellow());
        println!("\n{}", "Что нужно сделать:".cyan());
        println!("  1. Перейдите в меню 'Регистры' (пункт 4)");
        println!("  2. Выберите 'Добавить регистр' (пункт 3)");
        println!("  3. Добавьте необходимые регистры для опроса");
        wait_for_continue()?;
        return Ok(());
    }

    let enabled_registers: Vec<&RegisterConfig> = registers_config
        .registers
        .iter()
        .filter(|reg| reg.enabled)
        .collect();

    if enabled_registers.is_empty() {
        println!("\n{}", "ОШИБКА: Нет активных регистров для опроса!".red().bold());
        println!(
            "{}",
            format!(
                "В файле tags.csv есть {} регистр(ов), но все они отключены (enabled: false).",
                registers_config.registers.len()
            )
            .yellow()
        );
        println!("\n{}", "Что нужно сделать:".cyan());
        println!("  1. Перейдите в меню 'Регистры' (пункт 4)");
        println!("  2. Выберите 'Показать регистры' (пункт 1) - посмотрите список");
        println!("  3. Отредактируйте файл tags.csv и установите enabled: true для нужных регистров");
        println!("     или добавьте новые регистры через 'Добавить регистр' (пункт 3)");
        wait_for_continue()?;
        return Ok(());
    }

    println!("Используемые настройки подключения:");
    println!("  Режим: {}", conn.mode.bright_white());
    println!(
        "  Адрес устройства: {}",
        conn.device_address.to_string().bright_white()
    );

    if is_tcp_mode(&conn.mode) {
        println!("  TCP хост: {}", conn.tcp_host.bright_white());
        println!("  TCP порт: {}", conn.tcp_port.to_string().bright_white());
    } else {
        println!("  COM-порт: {}", conn.port.bright_white());
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
    }

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

    let slave_addr = Slave(conn.device_address);

    if is_tcp_mode(&conn.mode) {
        let socket_text = format!("{}:{}", conn.tcp_host, conn.tcp_port);
        let socket_addr = socket_text
            .to_socket_addrs()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Не удалось разрешить TCP адрес подключения",
                )
            })?;

        let mut ctx = match tokio_modbus::client::tcp::connect(socket_addr).await {
            Ok(ctx) => {
                println!(
                    "{}",
                    format!("TCP соединение с {} успешно установлено", socket_text).green()
                );
                ctx
            }
            Err(e) => {
                eprintln!("{}", format!("Ошибка TCP подключения: {:?}", e).red());
                return Err(io::Error::other(format!("{:?}", e)));
            }
        };

        ctx.set_slave(slave_addr);
        run_polling_loop(&mut ctx, &enabled_registers).await?;
    } else {
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

        let builder = tokio_serial::new(&conn.port, conn.baud_rate)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(parity)
            .stop_bits(stop_bits);

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
                return Err(io::Error::other(format!("{:?}", e)));
            }
        };

        ctx.set_slave(slave_addr);
        run_polling_loop(&mut ctx, &enabled_registers).await?;
    }

    Ok(())
}
