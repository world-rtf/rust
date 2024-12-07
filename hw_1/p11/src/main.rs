use std::io;
fn main() {
    loop {
        println!("input (fc - fahrenheit to celsuis, cf - celsuis to fahrenheit, end - break)");
        let mut z = String::new();
        io::stdin().read_line(&mut z).unwrap();

        if z.trim() == "fc" {
            println!("Input (F - fahrenheit):");
            let mut f = String::new();
            io::stdin().read_line(&mut f).unwrap();
            let f: f32 = match f.trim().parse() {
                Ok(num) => num,
                Err(_) => continue,
            };
            println!("{} C -> {} F ", f, (f * 9.0) / 5.0 + 32.0);
        }

        if  z.trim() == "cf"{
            println!("Input (C - celsuis):");
            let mut f = String::new();
            io::stdin().read_line(&mut f).unwrap();
            let f: f32 = match f.trim().parse() {
                Ok(num) => num,
                Err(_) => continue,
            };
            println!("{} F -> {} C ", f, (f - 32.0) * 5.0 /9.0);
        }

        if z.trim() == "end" {
            break;
        }
    }
}
