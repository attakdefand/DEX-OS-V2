// Example Rust code with various errors for testing the debugging server

fn main() {
    let x = String::from("hello");
    let y = &x;
    println!("{}", y);
    
    // Type mismatch error
    let result: String = add_numbers(5, 10);
    println!("Result: {}", result);
    
    // Borrow checker error
    let message = get_message();
    let message_ref = &message;
    use_message(message); // This moves message
    println!("Message: {}", message_ref); // This tries to use moved value
}

fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

fn get_message() -> String {
    String::from("Hello, world!")
}

fn use_message(msg: String) {
    println!("Using message: {}", msg);
}

// Lifetime error
fn get_first_word(s: &str) -> &str {
    let words: Vec<&str> = s.split_whitespace().collect();
    &words[0] // Lifetime error - returning reference to local data
}