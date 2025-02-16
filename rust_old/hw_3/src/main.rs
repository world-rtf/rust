use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let file = fs::read_to_string("file.txt")?;
    let lines: Vec<&str> = file.trim().split('\n').collect();
    
    let mut data: Vec<Vec<&str>> = Vec::new();
    for line in lines {
        let row: Vec<&str> = line.split_whitespace().collect();
        data.push(row);
    }
    
    let mut transposed: Vec<Vec<&str>> = vec![Vec::new(); data[0].len()];
    for i in 0..data.len() {
        for j in 0..data[0].len() {
            transposed[j].push(data[i][j]);
        }
    }
    
    for row in transposed {
        println!("{}", row.join(" "));
    }

    Ok(())
}
