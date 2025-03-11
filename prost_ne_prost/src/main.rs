use std::fs;
use std::collections::HashMap;
use std::error::Error;
use std::io::{BufReader, Read, Write, BufWriter};
use prost::Message;
use prost_types::Timestamp;
use std::time;
use std::{env, process};

mod pb {
    include!(concat!(env!("OUT_DIR"), "/addressbook.rs"));
}

const DB_FILE_PATH: &str = "addressbook.db";

struct Config {
    command: String,
    params: HashMap<String, String>,
}

impl Config {
    // Метод для построения конфигурации из аргументов командной строки
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Config, &'static str> {
        args.next(); // Пропускаем имя программы

        let command = args.next().ok_or("Command not found")?;

        let mut params: HashMap<String, String> = HashMap::new();
        let mut redact = false;

        while let Some(arg) = args.next() {
            if arg.starts_with("--") {
                // установка флага редактирования
                if arg == "--redact" {
                    redact = true; // Устанавливаем флаг редактирования
                } else {
                    let param = args.next().ok_or("Missing parameter after --arg")?;
                    params.insert(arg, param);
                }
            } else {
                return Err("Expected arg starts with --");
            }
        }

        // Добавляем флаг в параметры
        if redact {
            params.insert("--redact".to_string(), "true".to_string());
        }
        //
        Ok(Config { command, params })
    }
}

// открытие дб
fn open_db_file(file_path: &str) -> fs::File {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .unwrap()
}

fn read_from_db(f: &mut fs::File) -> pb::AddressBook {
    let mut buf_reader = BufReader::new(f); // Создаем буферизованный читател
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    pb::AddressBook::decode(contents.as_slice()).unwrap() // Декодируем содержимое в AddressBook
}

fn write_to_db(f: &mut fs::File, book: pb::AddressBook) {
    let mut buf_writer = BufWriter::new(f);
    let contents = book.encode_to_vec(); // Кодируем AddressBook в вектор байтов
    buf_writer.write_all(&contents).unwrap(); // запись
    buf_writer.flush().unwrap(); // сброс буфера
}

fn str_to_phone_type(s: &str) -> i32 {
    match s {
        "home" => 2,
        "mobile" => 1,
        "work" => 3,
        _ => 0,
    }
}

fn str_to_department(s: &str) -> i32 {
    match s {
        "hr" => 1,
        "cs" => 2,
        _ => 0,
    }
}

fn add_person(f: &mut fs::File, name: &str, email: &str, phone: &str, phone_type: &str) {
    let mut book = read_from_db(f);
    let mut person = pb::Person::default(); // создание экземпляра по умолчанию

    // установка аргументов
    person.email = email.to_string();
    let mut nb = pb::person::PhoneNumber::default();
    nb.number = phone.to_string();
    nb.r#type = str_to_phone_type(phone_type);
    person.phones.push(nb);

    // Создаем новый контакт
    let mut contact = pb::Contact::default();
    let mut update_ts = Timestamp::default(); // Создаем временную метку
    let duration = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
    update_ts.seconds = duration.as_secs() as i64;
    update_ts.nanos = duration.subsec_nanos() as i32;

    contact.last_updated = Some(update_ts); // установка времени обновление
    contact.kind = Some(pb::contact::Kind::Person(person)); // Тип
    book.contacts.insert(name.to_string(), contact); // Добавление в книгу

    write_to_db(f, book);
}

fn add_company(f: &mut fs::File, name: &str, email: &str, email_dep: &str, phone: &str, phone_dep: &str) {
    let mut book = read_from_db(f);
    let mut company = pb::Company::default();

    let mut addr = pb::company::EmailAddress::default();
    addr.email = email.to_string();
    addr.department = str_to_department(email_dep);
    company.emails.push(addr);

    let mut nb = pb::company::PhoneNumber::default();
    nb.number = phone.to_string();
    nb.department = str_to_department(phone_dep);
    company.phones.push(nb);

    let mut contact = pb::Contact::default();
    let mut update_ts = Timestamp::default();
    let duration = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
    update_ts.seconds = duration.as_secs() as i64;
    update_ts.nanos = duration.subsec_nanos() as i32;

    contact.last_updated = Some(update_ts);
    contact.kind = Some(pb::contact::Kind::Company(company));
    book.contacts.insert(name.to_string(), contact);

    write_to_db(f, book);
}

fn redact_private_info(contact: &mut pb::Contact) {
    // человек
    if let Some(pb::contact::Kind::Person(ref mut person)) = contact.kind {
        // email
        person.email = "*".repeat(person.email.len());
        
        //  телефон
        for phone in person.phones.iter_mut() {
            phone.number = "*".repeat(phone.number.len());
        }
    }
    // компания
    else if let Some(pb::contact::Kind::Company(ref mut company)) = contact.kind {
        for email in company.emails.iter_mut() {
            email.email = "*".repeat(email.email.len());
        }

        for phone in company.phones.iter_mut() {
            phone.number = "*".repeat(phone.number.len());
        }
    }
}


fn list_contacts(f: &mut fs::File, redact: bool) {
    let book = read_from_db(f);
    let mut keys: Vec<&String> = book.contacts.keys().collect();
    keys.sort();
    for name in keys {
        let mut contact = book.contacts.get(name).unwrap().clone(); // Клонируем контакт для редактирования
        if redact {
            redact_private_info(&mut contact); // Редактируем личную информацию
        }
        println!("name: {}", name);
        println!("last_updated: {:?}", contact.last_updated.unwrap());
        println!("{:#?}", contact);
        println!("-----------------------");
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut f = open_db_file(DB_FILE_PATH);
    match config.command.as_ref() {
        "add" => {
            if config.params["--kind"] == "per" || config.params["--kind"] == "person" {
                add_person(
                    &mut f,
                    &config.params["--name"],
                    &config.params["--email"],
                    &config.params["--phone"],
                    &config.params["--type"],
                );
            } else if config.params["--kind"] == "cie" || config.params["--kind"] == "company" {
                add_company(
                    &mut f,
                    &config.params["--name"],
                    &config.params["--email"],
                    &config.params["--dep"],
                    &config.params["--phone"],
                    &config.params["--type"],
                );
            }
            Ok(())
        }

        "list" => {
            let redact = config.params.get("--redact").is_some();
            list_contacts(&mut f, redact);
            Ok(())
        }
        _ => Err("Command not found")?,
    }
}

fn main() {
    let config = Config::build(env::args()).expect("Build Error");

    if let Err(e) = run(config) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}