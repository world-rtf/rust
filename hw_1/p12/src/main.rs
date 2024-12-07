use std::io;

fn fib(n: u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let t = a;
        a=b;
        b = t + b;
    }
    return a
}

fn main() {
    println!("Введите число n:");

    let mut f = String::new();
    io::stdin()
        .read_line(&mut f)
        .unwrap();
    let n: u32 = f.trim().parse().unwrap();
    let result = fib(n);
    println!("n-е число фибаначи {}", result);
}
