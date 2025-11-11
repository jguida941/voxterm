#!/usr/bin/env rust-script
//! Test to reproduce the ratatui panic

use std::io::{self, Write};

fn main() {
    println!("Testing potential panic scenarios...\n");

    // Test 1: Create a string that would cause issues at position 500
    let test1 = "a".repeat(499) + "世界";
    println!("Test 1 - Multi-byte at boundary 500:");
    println!("  String length: {} bytes, {} chars", test1.len(), test1.chars().count());

    // Simulate what ratatui might do - this is where the panic could happen
    // if there's a width calculation error
    simulate_wrapping(&test1, 500);

    // Test 2: String with zero-width characters
    let test2 = "Normal text\u{200B}zero-width\u{FEFF}here";
    println!("\nTest 2 - Zero-width characters:");
    simulate_wrapping(&test2, 20);

    // Test 3: Very long line with emojis
    let test3 = "Start " + &"🦀".repeat(250) + " End";
    println!("\nTest 3 - Many emojis:");
    simulate_wrapping(&test3, 500);

    // Test 4: Terminal control sequences
    let test4 = "\x1b[31mColored\x1b[0m text with \x1b[1;1R cursor report";
    println!("\nTest 4 - Control sequences:");
    simulate_wrapping(&test4, 50);

    // Test 5: The specific panic index as a test
    let huge_index = 18446638520560386049usize;
    println!("\nTest 5 - Huge index (from panic):");
    println!("  Index: {}", huge_index);
    println!("  As signed i64: {}", huge_index as i64);
    println!("  Likely from: {} - {}", 0i64, huge_index as i64);

    // This would be approximately: 0 - 105524685523967
    // which suggests cursor=0, width=105524685523967 or similar

    println!("\n✅ Tests completed without panic!");
}

fn simulate_wrapping(text: &str, max_width: usize) {
    use unicode_width::UnicodeWidthStr;

    let width = UnicodeWidthStr::width(text);
    println!("  Display width: {}, max: {}", width, max_width);

    // Simulate what might cause the panic:
    // 1. Calculate start position (this is where underflow might happen)
    let cursor = 0usize;
    let viewport_width = max_width;

    // BAD: This would underflow if cursor < viewport_width/2
    // let start = cursor - viewport_width / 2;  // DON'T DO THIS!

    // GOOD: Use saturating arithmetic
    let start = cursor.saturating_sub(viewport_width / 2);
    let end = start.saturating_add(viewport_width);

    println!("  Viewport: start={}, end={}", start, end);

    // Try to slice (safely)
    if let Some(c) = text.chars().nth(start) {
        println!("  Char at start: {:?}", c);
    }

    // Check if we can create a slice
    let mut char_start = 0;
    let mut char_end = text.len();
    let mut char_count = 0;

    for (idx, _) in text.char_indices() {
        if char_count == start {
            char_start = idx;
        }
        if char_count == end {
            char_end = idx;
            break;
        }
        char_count += 1;
    }

    if char_start <= char_end && char_end <= text.len() {
        let slice = &text[char_start..char_end.min(text.len())];
        println!("  Safe slice: {} chars", slice.chars().count());
    }
}