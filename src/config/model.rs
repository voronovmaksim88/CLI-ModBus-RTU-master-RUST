use serde::{Deserialize, Serialize};

use crate::config::constants::{DEFAULT_TCP_HOST, DEFAULT_TCP_PORT, MODE_RTU, MODE_TCP};

/// Структура для хранения настроек подключения
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionSettings {
    pub port: String,
    pub device_address: u8,
    pub baud_rate: u32,
    pub parity: String,
    pub stop_bits: u8,
    #[serde(default = "default_connection_mode")]
    pub mode: String,
    #[serde(default = "default_tcp_host")]
    pub tcp_host: String,
    #[serde(default = "default_tcp_port")]
    pub tcp_port: u16,
}

/// Структура для метаданных
#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub last_updated: String,
    pub version: String,
    pub description: String,
}

/// Структура для описания регистра
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterConfig {
    pub name: String,
    pub description: String,
    pub address: u16,
    pub var_type: String,
    pub modbus_type: String,
    pub enabled: bool,
}

/// Структура для хранения всех регистров
#[derive(Serialize, Deserialize, Debug)]
pub struct RegistersConfig {
    pub registers: Vec<RegisterConfig>,
    pub metadata: Metadata,
}

/// Основная структура конфигурации
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub connection: ConnectionSettings,
    pub metadata: Metadata,
}

pub fn default_connection_mode() -> String {
    MODE_RTU.to_string()
}

pub fn default_tcp_host() -> String {
    DEFAULT_TCP_HOST.to_string()
}

pub fn default_tcp_port() -> u16 {
    DEFAULT_TCP_PORT
}

pub fn is_tcp_mode(mode: &str) -> bool {
    mode.eq_ignore_ascii_case(MODE_TCP)
}
