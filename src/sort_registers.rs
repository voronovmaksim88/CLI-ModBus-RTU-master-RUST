use std::io;
use colored::*;

use crate::{
	load_registers,
	save_registers_to_csv,
	RegistersConfig,
};

#[allow(dead_code)]
/// Читает все регистры из CSV, сортирует по возрастанию адреса и сохраняет обратно
pub fn sort_registers_by_address() -> io::Result<()> {
	let mut cfg: RegistersConfig = load_registers()?;

	// Сортировка по адресу по возрастанию
	cfg.registers.sort_by_key(|reg| reg.address);

	// Перезапись CSV
	save_registers_to_csv(&cfg.registers)?;

	println!("{}", "Регистры отсортированы по адресу и сохранены в tags.csv".green());
	Ok(())
}


