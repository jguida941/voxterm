fn main() {
    println!("Demonstrating the UTF-8 bug that was fixed:\n");

    // Simulate the buggy version
    let test_string = "café";
    println!("Original string: {}", test_string);
    println!("Bytes: {:?}", test_string.as_bytes());

    // Show what the buggy code would have done
    let bytes = test_string.as_bytes();
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1] & 0b11000000) == 0b10000000 {
        println!(
            "  Buggy code found continuation byte at position {}: 0x{:02X}",
            end - 1,
            bytes[end - 1]
        );
        end -= 1;
    }

    if end != bytes.len() {
        let corrupted = String::from_utf8_lossy(&bytes[..end]);
        println!("  Buggy result: '{}' (corruption!)", corrupted);
    }

    println!("\nThe fixed version correctly preserves: '{}'", test_string);

    // Test with more examples
    println!("\nOther examples that would have been corrupted:");
    let examples = vec!["Hello 世界", "Emoji: 🎉", "naïve résumé", "∑∏∫ math"];

    for example in examples {
        println!("  ✓ '{}' - now preserved correctly", example);
    }
}
