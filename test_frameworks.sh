#!/bin/bash
set -e

# Path to the newly built devbind CLI
DEVBIND="$(pwd)/target/debug/devbind-cli"

if [ ! -f "$DEVBIND" ]; then
    echo "❌ devbind-cli not found. Please run 'cargo build -p devbind-cli' first."
    exit 1
fi

TESTDIR=$(mktemp -d)
echo "🚀 Creating test isolated workspace at $TESTDIR"

function test_app() {
    local name=$1
    local setup_cmd=$2
    # Use array for run cmd to avoid word splitting issues
    shift 2
    local run_cmd=("$@")

    echo ""
    echo "======================================"
    echo "Testing $name"
    echo "======================================"

    APP_DIR="$TESTDIR/$name"
    mkdir -p "$APP_DIR"
    cd "$APP_DIR"

    echo "=> Setting up $name app..."
    eval "$setup_cmd"

    echo "=> Running 'devbind run $name ${run_cmd[*]}'..."

    # Run in background without eval (so literal $PORT gets passed)
    "$DEVBIND" run "$name" "${run_cmd[@]}" > devbind.log 2>&1 &
    local PID=$!

    # Wait for devbind to allocate a port and print it
    for i in {1..20}; do
        if grep -q "http://127.0.0.1:" devbind.log 2>/dev/null; then
            break
        fi
        sleep 0.5
    done

    # Extract port from log: "🔗  myapp.local → http://127.0.0.1:45321"
    PORT=$(grep -oP "(?<=http://127.0.0.1:)\d+" devbind.log || echo "")

    if [ -z "$PORT" ]; then
        echo "❌ Failed to extract port from devbind output. Log:"
        cat devbind.log
        kill -9 $PID 2>/dev/null || true
        exit 1
    fi

    echo "=> Background PID: $PID, Assigned Port: $PORT"
    echo "=> Testing connection to http://$name.local:$PORT..."

    # Give the app a few seconds to boot up and bind to the port
    local success=false
    for i in {1..20}; do
        if curl -s -m 2 "http://$name.local:$PORT" > /dev/null; then
            success=true
            break
        fi
        sleep 0.5
    done

    if [ "$success" = true ]; then
        echo "✅ Success! $name app responded."
    else
        echo "❌ Failed to connect to $name app."
        echo "--- Log Output ---"
        cat devbind.log
        kill -9 $PID 2>/dev/null || true
        exit 1
    fi

    echo "=> Cleaning up $name..."
    kill -INT $PID 2>/dev/null || true
    wait $PID 2>/dev/null || true
    echo "✅ OK"
}

# 1. Python HTTP Server
if command -v python3 &> /dev/null; then
    test_app "pythonapp" "echo 'Hello Python' > index.html" python3 -m http.server '$PORT'
else
    echo "⏭️  Skipping Python test (python3 not installed)"
fi

# 2. PHP Built-in Server
if command -v php &> /dev/null; then
    test_app "phpapp" "echo '<?php echo \"Hello PHP\"; ?>' > index.php" php -S '0.0.0.0:$PORT'
else
    echo "⏭️  Skipping PHP test (php not installed)"
fi

# 3. Node.js (Raw HTTP Server)
if command -v node &> /dev/null; then
    NODE_SETUP="cat << 'EOF' > server.js
const http = require('http');
const port = process.env.PORT || 3000;
http.createServer((req, res) => { res.end('Hello Node'); }).listen(port);
EOF"
    test_app "nodeapp" "$NODE_SETUP" node server.js
else
    echo "⏭️  Skipping Node.js test (node not installed)"
fi

# 4. React Vite App (Node.js)
if command -v npm &> /dev/null; then
    VITE_SETUP="npm create vite@latest --yes . -- --template react > /dev/null 2>&1 && npm install > /dev/null 2>&1"
    test_app "viteapp" "$VITE_SETUP" npm run dev -- --port '$PORT' --host
else
    echo "⏭️  Skipping React Vite test (npm not installed)"
fi

echo ""
echo "🎉 All framework integration tests passed successfully!"
rm -rf "$TESTDIR"
