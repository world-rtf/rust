use std::collections::HashMap;
use std::io;

fn main() {
    let mut input: String = String::new();
    println!("Input:");
    io::stdin().read_line(&mut input).expect("User ERROR");

    let lst: Vec<i32> = input
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    let sumlst: i32 = lst.iter().sum();
    let mean = sumlst as f64 / lst.len() as f64;

    let mut sorted_lst = lst.clone();
    sorted_lst.sort();
    let median = if lst.len() % 2 == 0 {
        (sorted_lst[lst.len() / 2 - 1] + sorted_lst[lst.len() / 2]) as f64 / 2.0
    } else {
        sorted_lst[lst.len() / 2] as f64
    };

    let mut smoda = HashMap::new();
    for &num in &sorted_lst {
        *smoda.entry(num).or_insert(0) +=1;
    };
    let moda =smoda.into_iter().max_by_key(|&(_, count)| count).map(|(number, _)| number); // игнор второго значения _

    println!("Среднее: {mean}");
    println!("Медиана: {median}");
    // гемор
    match moda {
        Some(m) => println!("Мода: {m}"),
        None => println!("Usser Error"),
    }

}
