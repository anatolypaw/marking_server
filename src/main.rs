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
    let mut bd_client = match Client::connect(dsn, NoTls) {
        Ok(bd_client) => {
            println!(
                "{} {} {} {}",
                "###".green(),
                Local::now(),
                "Connected to".green(),
                dsn
            );
            bd_client
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
        let message_type = str_between(&datagram, "", "\n");
        print!("{} Type: {}", "???".cyan(), message_type);

        match message_type {
            //АВ передала свой статус
            "aw_status" => {
                let aw_name = str_between(&datagram, "aw_name: ", "\n");
                let ip = str_between(&datagram, "ip: ", "\n");
                println!("\n{}\n{}", aw_name, ip);
            }

            //АВ запрашивает код маркировки
            "get_new_mc" => {
                let gtin = str_between(&datagram, "gtin: ", "\n");

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
fn str_between<'a>(source: &'a str, start: &'a str, end: &'a str) -> &'a str {
    let start_position = source.find(start);

    if start_position.is_some() {
        let start_position = start_position.unwrap() + start.len();
        let source = &source[start_position..];
        let end_position = source.find(end).unwrap_or_default();
        return &source[..end_position];
    }
    return "";
}
