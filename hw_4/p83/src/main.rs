use std::collections::HashMap;
use std::io;

fn main() {
    let mut company: HashMap<String, Vec<String>> = HashMap::new();

    loop {
        println!("Введите команду (add [name] to [department] | list [department] | list company) | exit:");
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        if command.starts_with("add ") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            let name = parts[1];
            let department = parts[3];
            company.entry(department.to_string()).or_insert(Vec::new()).push(name.to_string());
            println!("{} добавлен в {}", name, department);
        } else if command.starts_with("list ") {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.len() < 2 {
                println!("Неправильная команда");
                continue;
            }
            if parts[1] == "company" {
                let mut all_departments: Vec<_> = company.keys().collect();
                all_departments.sort();
                for department in all_departments {
                    println!("{department}: {:?}", company[department]);
                }
            } else {
                let department = parts[1];
                if let Some(staff) = company.get(department) {
                    println!("{:?}", staff);
                }
            }
        } else if command == "exit" {
            break;
        } else {
            println!("Неизвестная команда.");
        }
    }
}