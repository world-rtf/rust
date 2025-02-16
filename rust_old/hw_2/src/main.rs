fn main() {
    if num_rows == 1 {
    	return input_string;
	}

	let mut result = String::new();
	let cycle_length = 2 * num_rows - 2;

	for row in 0..num_rows {
	    let mut interval = if row == num_rows - 1 { cycle_length } else { 2 * (num_rows - row - 1) };
	    let mut index = row;
	    
	    while index < input_string.len() as i32 {
		result.push(input_string.chars().nth(index as usize).unwrap());
		index += interval;
		interval = if interval == cycle_length || interval == 0 {
		    cycle_length
		} else {
		    cycle_length - interval
		};
	    }
	}

	result
}
