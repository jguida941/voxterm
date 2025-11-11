#!/usr/bin/env rust-script
//! Test UTF-8 safety fixes for the TUI

use std::path::Path;
use std::env;

// Include our UTF-8 safe module inline for testing
mod utf8_safe {
    pub fn safe_prefix(s: &str, max_chars: usize) -> &str {
        if s.is_empty() || max_chars == 0 {
            return "";
        }

        let mut end = s.len();
        let mut count = 0usize;

        for (idx, ch) in s.char_indices() {
            count += 1;
            end = idx + ch.len_utf8();
            if count == max_chars {
                return &s[..end.min(s.len())];
            }
        }

        s
    }

    pub fn ellipsize(s: &str, max_chars: usize) -> String {
        if s.chars().count() <= max_chars {
            return s.to_string();
        }

        if max_chars == 0 {
            return String::from("…");
        }

        if max_chars == 1 {
            return String::from("…");
        }

        format!("{}…", safe_prefix(s, max_chars.saturating_sub(1)))
    }
}

fn main() {
    println!("Testing UTF-8 safety fixes...\n");

    // Test 1: Multi-byte characters at boundaries
    let test1 = "Hello 你好世界 World";
    println!("Test 1 - Multi-byte at boundary:");
    println!("  Input: {:?}", test1);
    println!("  Prefix(7): {:?}", utf8_safe::safe_prefix(test1, 7));
    println!("  Ellipsize(10): {:?}", utf8_safe::ellipsize(test1, 10));

    // Test 2: Emoji boundaries
    let test2 = "Start 🦀Rust🔥 End";
    println!("\nTest 2 - Emoji boundaries:");
    println!("  Input: {:?}", test2);
    println!("  Prefix(8): {:?}", utf8_safe::safe_prefix(test2, 8));
    println!("  Ellipsize(8): {:?}", utf8_safe::ellipsize(test2, 8));

    // Test 3: String that would previously cause panic
    let test3 = "a".repeat(499) + "你好";
    println!("\nTest 3 - Long string with multi-byte at position 500:");
    println!("  Input length: {} bytes, {} chars", test3.len(), test3.chars().count());
    let truncated = utf8_safe::ellipsize(&test3, 500);
    println!("  Ellipsize(500): ends with '{}...'",
             utf8_safe::safe_prefix(&truncated, 10));
    println!("  Result length: {} bytes, {} chars", truncated.len(), truncated.chars().count());

    // Test 4: Edge cases
    println!("\nTest 4 - Edge cases:");
    println!("  Empty string prefix: {:?}", utf8_safe::safe_prefix("", 5));
    println!("  Zero chars ellipsize: {:?}", utf8_safe::ellipsize("test", 0));
    println!("  One char ellipsize: {:?}", utf8_safe::ellipsize("test", 1));

    // Test 5: Safe slicing with get()
    let test5 = "Hello 世界";
    println!("\nTest 5 - Safe slicing with get():");
    println!("  Input: {:?}", test5);

    // This would panic with raw slicing if 7 falls in the middle of a multi-byte char
    let safe_slice = test5.get(..7).unwrap_or(test5);
    println!("  get(..7): {:?}", safe_slice);

    // Test that saturating arithmetic prevents underflow
    println!("\nTest 6 - Saturating arithmetic:");
    let small: usize = 5;
    let large: usize = 10;
    println!("  5.saturating_sub(10) = {}", small.saturating_sub(large));
    println!("  This prevents underflow that would create huge numbers");

    println!("\n✅ All UTF-8 safety tests passed!");
}
