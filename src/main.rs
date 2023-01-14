use chrono::Local;
use colored::Colorize;
use postgres::{Client, NoTls};
use std::time::Instant;
use std::{net::UdpSocket, str};

fn main() {
    println!(
        "{} {} {}",
        "###".green(),
        Local::now(),
        "Server start".green().bold()
    );

    //Создаем сокет, для обмена данными с automated workstation
    let socket = match UdpSocket::bind("0.0.0.0:4700") {
        Ok(socket) => {
            println!(
                "{} {} {} {}",
                "###".green(),
                Local::now(),
                "UDP Socket at".green(),
                socket.local_addr().unwrap()
            );
            socket
        }
        Err(e) => {
            println!("{} {}", "@@@".red(), e);
            return;
        }
    };

    //Подключаемся к базе данных
    let dsn = "postgresql://test:test@localhost/test_bd";
    let mut pg_client = match Client::connect(dsn, NoTls) {
        Ok(pg_client) => {
            println!(
                "{} {} {} {}",
                "###".green(),
                Local::now(),
                "Connected to".green(),
                dsn
            );
            pg_client
        }
        Err(e) => {
            let err = format!(
                "{} {} {} {:?}",
                "@@@".red(),
                Local::now(),
                "Postgresql connection error:".red(),
                e
            );
            println!("{}", err);
            return;
        }
    };

    loop {
        //Принимаем пакет, сохраняем данные в buf
        let mut datagram = [0; 255];
        let (number_of_bytes, src_addr) = socket
            .recv_from(&mut datagram)
            .expect("Didn't receive data");

        //Запускаем счетчик времени, для понимания сколько занимает обработка
        let timer_start = Instant::now();

        //Сохраняем из буфера только полученное количество байт
        let datagram = &mut datagram[..number_of_bytes];
        let datagram = str::from_utf8(&datagram).unwrap();

        //Выводим сообщение о том что принят пакет
        println!(
            "\n{} {} {} {src_addr}",
            ">>>".magenta(),
            Local::now(),
            "Received message from".magenta()
        );
        println!("{}", &datagram);

        //Обрабатываем принятую датаграмму
        //Считываем первую строку сообщения, она должна содержать тип сообщения
        let message_type = get_value_by_key(&datagram, "");
        print!("{} Type: {}", "???".cyan(), message_type);

        match message_type {
            //АВ передала свой статус
            "aw_status" => {
                let aw_name = get_value_by_key(&datagram, "aw_name: ");
                let ip = get_value_by_key(&datagram, "ip: ");
                println!("\n{}\n{}", aw_name, ip);

                let c = pg_client
                    .query_one("SELECT * FROM aw", &[])
                    .expect("failed to query item");
                
                let quantity: i32 = c.get("aw_id");
                let name: String = c.get("aw_name");

                println!("id {} name {}", name, quantity);
                
            }

            //АВ запрашивает код маркировки
            "get_new_mc" => {
                let gtin = get_value_by_key(&datagram, "gtin: ");

                //Проверяем корректность запрошенного gtin
                if gtin.len() == 14 {
                    println!("Запрошен gtin {}", gtin);
                } else {
                    println!(
                        "{} {} {}",
                        "@@@".red(),
                        Local::now(),
                        "Длина gtin некорректна".red()
                    );
                }
            }
            _ => {
                println!("{}", "NOT RECOGNIZED".red());
            }
        }

        //Вычисляем прошедшее время
        println!("{} Timer elapsed {:?}", "???".cyan(), timer_start.elapsed());
    }
}

//Выводит значение ключа
fn get_value_by_key<'a>(source: &'a str, key: &'a str) -> &'a str {
    let start_position = source.find(key);

    if start_position.is_some() {
        let start_position = start_position.unwrap() + key.len();
        let source = &source[start_position..];
        let end_position = source.find("\n").unwrap_or_default();
        return &source[..end_position];
    }
    return "";
}
