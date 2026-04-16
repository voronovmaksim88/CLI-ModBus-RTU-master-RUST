use colored::*;
use std::io::{self, Write};

#[cfg(windows)]
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
#[cfg(windows)]
use winapi::um::processenv::GetStdHandle;
#[cfg(windows)]
use winapi::um::winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
#[cfg(windows)]
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

/// Включение поддержки цветного вывода в Windows
#[cfg(windows)]
pub fn enable_ansi_support() {
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
pub fn enable_ansi_support() {}

/// Функция очистки экрана консоли
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Функция ожидания нажатия Enter для продолжения
pub fn wait_for_continue() -> io::Result<()> {
    println!("\n{}", "Нажмите Enter для продолжения...".bright_black());
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}
