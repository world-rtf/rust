fn main() {
    let text = [
        "A partridge in a pear tree",
        "Two turtle doves",
        "Three French hens",
        "Four calling birds",
        "Five golden rings",
        "Six geese-a-laying",
        "Seven swans-a-swimming",
        "Eight maids-a-milking",
        "Nine drummers drumming",
        "Ten pipers piping",
        "Eleven ladies dancing",
        "Twelve lords-a-leaping",
    ];

    let num = [
        "first", "second", "third", "fourth", "fifth", "sixth",
        "seventh", "eighth", "ninth", "tenth", "eleventh", "twelfth",
    ];

    for i in 0..12 {
        println!("On the {} day of Christmas", num[i]);
        println!("My true love gave to me");

        for j in (0..=i).rev() {
            if i > 0 && j == 0 {
                println!("And {}", text[j]);
            } 
            else {
                println!("{}", text[j]);
            }
        }
        println!();
    }
}
