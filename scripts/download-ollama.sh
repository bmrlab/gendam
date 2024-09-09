# bash scripts/download-ollama.sh

ollama show qwen2:7b-instruct-q4_0 > /dev/null 2>&1 || ollama pull qwen2:7b-instruct-q4_0
ollama show llava-phi3:3.8b-mini-q4_0 > /dev/null 2>&1 || ollama pull llava-phi3:3.8b-mini-q4_0
