use colored::*;

pub fn scan_available_ports(available_ports: &mut [u8; 10]) -> usize {
    let mut count = 0;

    println!("Сканирование доступных COM-портов...");

    match serialport::available_ports() {
        Ok(ports) => {
            for port in ports {
                if let Some(port_name) = port.port_name.strip_prefix("COM") {
                    if let Ok(port_num) = port_name.parse::<u8>() {
                        if count < 10 {
                            available_ports[count] = port_num;
                            println!("  Найден: COM{}", port_num);
                            count += 1;
                        }
                    }
                }
            }

            if count == 0 {
                println!("{}", "  COM-порты не найдены".yellow());
            } else {
                println!("{}", format!("  Всего найдено портов: {}", count).cyan());
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Ошибка сканирования портов: {:?}", e).red());
        }
    }

    count
}
