pub mod add_register;
pub mod config;
pub mod modbus;
pub mod paths;
pub mod persistence;
pub mod prompts;
pub mod register;
pub mod scan_available_ports;
pub mod sort_registers;
pub mod terminal;
pub mod ui;

use colored::*;
use std::io;

use add_register::add_register;
use modbus::poll::start_polling;
use persistence::registers_csv::initialize_registers_if_needed;
use persistence::settings_json::initialize_config_if_needed;
use terminal::wait_for_continue;
use ui::connection::{change_connection_settings, show_connection_settings, switch_connection_mode};
use ui::main_menu::show_main_menu;
use ui::registers::{delete_all_registers, delete_register, show_registers, show_registers_menu};

pub async fn run() -> io::Result<()> {
    initialize_config_if_needed()?;
    initialize_registers_if_needed()?;

    loop {
        let choice = show_main_menu()?;

        match choice {
            1 => {
                show_connection_settings()?;
                wait_for_continue()?;
                println!();
            }
            2 => {
                change_connection_settings()?;
                wait_for_continue()?;
            }
            3 => match start_polling().await {
                Ok(()) => continue,
                Err(e) => {
                    eprintln!("{}", format!("Ошибка при опросе: {}", e).red());
                    wait_for_continue()?;
                }
            },
            4 => loop {
                let registers_choice = show_registers_menu()?;
                match registers_choice {
                    1 => {
                        show_registers()?;
                        wait_for_continue()?;
                    }
                    2 => delete_register()?,
                    3 => add_register()?,
                    4 => {
                        println!(
                            "{}",
                            "Функция 'Изменить регистр' пока не реализована".yellow()
                        );
                        wait_for_continue()?;
                    }
                    5 => {
                        match sort_registers::sort_registers_by_address() {
                            Ok(()) => println!("{}", "Сортировка завершена".green()),
                            Err(e) => eprintln!("{}", format!("Ошибка сортировки: {}", e).red()),
                        }
                        wait_for_continue()?;
                    }
                    6 => delete_all_registers()?,
                    9 => break,
                    _ => unreachable!(),
                }
            },
            5 => {
                if let Err(e) = switch_connection_mode() {
                    eprintln!("{}", format!("Ошибка переключения режима: {}", e).red());
                }
                wait_for_continue()?;
            }
            9 => {
                println!("{}", "Завершение программы...".yellow());
                return Ok(());
            }
            _ => unreachable!(),
        }
    }
}
