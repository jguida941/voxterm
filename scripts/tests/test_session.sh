#!/bin/bash
# Quick test of persistent session

echo "Testing persistent Codex session..."
echo ""
echo "Instructions:"
echo "1. Type 'hello' and press Enter"
echo "2. Type 'what did I just say?' and press Enter"
echo "3. If Codex remembers 'hello', the session is working!"
echo "4. Press Ctrl+C to exit"
echo ""
echo "Starting in 3 seconds..."
sleep 3

cd /Users/jguida941/new_github_projects/voxterm
./start.sh