use colored::*;
use std::io::{self, Write};

use crate::{
	clear_screen,
	wait_for_continue,
	load_registers_or_warn,
	save_registers_to_csv,
	RegisterConfig,
};

/// Добавление нового регистра (интерактивно)
pub fn add_register() -> io::Result<()> {
	clear_screen();
	println!("{}", "=== Добавление регистра ===".cyan().bold());

	// Имя
	print!("{} ", "Какое имя регистра? (name):".yellow());
	io::stdout().flush()?;
	let mut name = String::new();
	io::stdin().read_line(&mut name)?;
	let name = name.trim().to_string();
	if name.is_empty() {
		println!("{}", "Имя не может быть пустым".red());
		wait_for_continue()?;
		return Ok(());
	}

	// Описание
	print!("{} ", "Описание? (description):".yellow());
	io::stdout().flush()?;
	let mut description = String::new();
	io::stdin().read_line(&mut description)?;
	let description = description.trim().to_string();

	// Адрес
	print!("{} ", "Адрес в десятичном формате? (address):".yellow());
	io::stdout().flush()?;
	let mut address_str = String::new();
	io::stdin().read_line(&mut address_str)?;
	let address = match address_str.trim().parse::<u16>() {
		Ok(v) => v,
		Err(_) => {
			println!("{}", "Неверный адрес. Ожидалось число 0..65535".red());
			wait_for_continue()?;
			return Ok(());
		}
	};

	// Тип переменной
	println!("{}", "Тип переменной? (var_type)".yellow());
	println!("  1. bool");
	println!("  2. u16");
	println!("  3. i16");
	println!("  4. u32");
	println!("  5. i32");
	println!("  6. float");
	print!("Введите номер (1-6): ");
	io::stdout().flush()?;
	let mut var_choice = String::new();
	io::stdin().read_line(&mut var_choice)?;
	let var_type = match var_choice.trim() {
		"1" => "bool".to_string(),
		"2" => "u16".to_string(),
		"3" => "i16".to_string(),
		"4" => "u32".to_string(),
		"5" => "i32".to_string(),
		"6" => "float".to_string(),
		_ => {
			println!("{}", "Неверный выбор var_type".red());
			wait_for_continue()?;
			return Ok(());
		}
	};

    // Тип Modbus регистра
    println!("{}", "Тип Modbus регистра? (modbus_type)".yellow());
    println!("  1. input_register (ф-ция чтения 0x04)");
    println!("  2. holding_register (ф-ция чтения 0x03)");
    println!("  3. coil (ф-ция чтения 0x01)");
    println!("  4. discrete_input (ф-ция чтения 0x02)");
    print!("Введите номер (1-4): ");
    io::stdout().flush()?;
    let mut modbus_choice = String::new();
    io::stdin().read_line(&mut modbus_choice)?;
    let modbus_type = match modbus_choice.trim() {
        "1" => "input_register".to_string(),
        "2" => "holding_register".to_string(),
        "3" => "coil".to_string(),
        "4" => "discrete_input".to_string(),
        _ => {
            println!("{}", "Неверный выбор modbus_type".red());
            wait_for_continue()?;
            return Ok(());
        }
    };

	// enabled
	println!("{}", "Разрешено ли запрашивать этот регистр? (enabled)".yellow());
	println!("  1. Да (True)");
	println!("  2. Нет (False)");
	print!("Введите номер (1-2): ");
	io::stdout().flush()?;
	let mut enabled_choice = String::new();
	io::stdin().read_line(&mut enabled_choice)?;
	let enabled = match enabled_choice.trim().to_lowercase().as_str() {
		"1" | "да" | "y" | "yes" | "true" => true,
		"2" | "нет" | "n" | "no" | "false" => false,
		_ => {
			println!("{}", "Неверный выбор для enabled".red());
			wait_for_continue()?;
			return Ok(());
		}
	};

	// Загрузка текущих
	let mut cfg = match load_registers_or_warn() {
		Some(c) => c,
		None => {
			return Ok(());
		}
	};

	// Добавляем
	let new_reg = RegisterConfig {
		name,
		description,
		address,
		var_type,
		modbus_type,
		enabled,
	};
	cfg.registers.push(new_reg);
	save_registers_to_csv(&cfg.registers)?;
	println!("{}", "Регистр добавлен".green());
	wait_for_continue()?;
	Ok(())
}


