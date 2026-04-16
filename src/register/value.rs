use crate::config::model::RegisterConfig;

/// Функция для обработки данных регистра в зависимости от типа
pub fn process_register_data(data: &[u16], register: &RegisterConfig) -> String {
    match register.var_type.as_str() {
        "bool" => {
            if data.len() >= 1 {
                if data[0] != 0 {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "u16" => {
            if !data.is_empty() {
                format!("{}", data[0])
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "i16" => {
            if !data.is_empty() {
                format!("{}", data[0] as i16)
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "u32" | "i32" => {
            if data.len() >= 2 {
                let high_word = data[1] as u32;
                let low_word = data[0] as u32;
                let combined = (high_word << 16) | low_word;
                if register.var_type == "i32" {
                    format!("{}", combined as i32)
                } else {
                    format!("{}", combined)
                }
            } else {
                "Недостаточно данных".to_string()
            }
        }
        "float" => {
            if data.len() >= 2 {
                let high_word = data[1] as u32;
                let low_word = data[0] as u32;
                let combined = (high_word << 16) | low_word;
                format!("{:.3}", f32::from_bits(combined))
            } else {
                "Недостаточно данных".to_string()
            }
        }
        _ => {
            let values: Vec<String> = data.iter().map(|&x| format!("{}", x)).collect();
            format!("[{}]", values.join(", "))
        }
    }
}

/// Вычисляет количество 16-битных регистров для чтения по типу переменной
pub fn compute_quantity(var_type: &str) -> u16 {
    match var_type {
        "bool" | "u16" | "i16" => 1,
        "u32" | "i32" | "float" => 2,
        _ => 1,
    }
}

/// Форматирование значения для булевых регистров (coil/discrete_input)
pub fn format_bool_value(data: Vec<bool>) -> String {
    if data.first().copied().unwrap_or(false) {
        "true".to_string()
    } else {
        "false".to_string()
    }
}
