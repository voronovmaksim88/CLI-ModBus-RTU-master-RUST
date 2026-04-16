/// Доступные скорости передачи данных для RS-485 (в бодах)
pub const AVAILABLE_BAUD_RATES: (u32, u32, u32, u32, u32, u32, u32) =
    (2400, 4800, 9600, 19200, 38400, 57600, 115200);

/// Доступные варианты четности для RS-485
pub const PARITY_OPTIONS: (&str, &str, &str) = ("None", "Even", "Odd");

/// Доступные варианты стоп-битов для RS-485
pub const STOP_BITS_OPTIONS: (&str, &str) = ("1 стоп-бит", "2 стоп-бита");

pub const MODE_RTU: &str = "RTU";
pub const MODE_TCP: &str = "TCP";
pub const DEFAULT_TCP_HOST: &str = "127.0.0.1";
pub const DEFAULT_TCP_PORT: u16 = 502;
