fn main() {
    let sentence = "first apple hello world";

    let pig_latin = sentence.split_whitespace()
        .map(|word| {
            let first_char = word.chars().next().unwrap();
            if "aeiouAEIOU".contains(first_char) {
                format!("{}-hay", word)
            } else {
                let mut chars = word.chars();
                let first = chars.next().unwrap();
                let rest: String = chars.collect();
                format!("{}-{}ay", rest, first)
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    println!("Порося латынь: {}", pig_latin);
}
