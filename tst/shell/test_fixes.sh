#!/usr/bin/env bash
# Test script to verify the formatting and permission fixes

echo "Testing Codex Voice Fixes"
echo "========================="
echo ""
echo "This test will verify:"
echo "1. Output formatting has proper newlines"
echo "2. Codex has full file write permissions"
echo ""
echo "Test Instructions:"
echo "------------------"
echo "1. Start the app: ./voice"
echo "2. Press Ctrl+R to record voice"
echo "3. Say: 'Create a file called test_file.py with a hello world function'"
echo "4. Press Enter to send"
echo ""
echo "Expected Results:"
echo "-----------------"
echo "✓ Your message appears with '>' prefix"
echo "✓ Blank line after your message"
echo "✓ Codex response appears"
echo "✓ Blank line after Codex response"
echo "✓ File test_file.py is created successfully"
echo ""
echo "Press Enter to start the test..."
read

# Run the voice app
exec ./voice