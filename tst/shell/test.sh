#!/bin/bash
# Test VOICE with persistent session

echo "=========================================="
echo "VOICE TEST - PERSISTENT SESSION"
echo "=========================================="
echo ""
echo "This tests if Codex remembers between VOICE inputs"
echo ""
echo "Instructions:"
echo "1. Press Ctrl+R → Speak: 'hello'"
echo "2. Press Enter to send"
echo "3. Press Ctrl+R → Speak: 'what did I just say?'"
echo "4. Press Enter to send"
echo "5. If Codex remembers 'hello' = PERSISTENT SESSION WORKS!"
echo ""
echo "Starting with REAL VOICE..."
echo ""

cd /Users/jguida941/new_github_projects/codex_voice

# Use the actual voice command with longer recording time
./voice -s 5