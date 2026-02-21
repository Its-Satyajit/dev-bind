#!/bin/bash
set -e

# Support user-local pnpm installation
export PATH="$HOME/.npm-global/bin:$PATH"

# Path to the build devbind CLI
DEVBIND="$(pwd)/target/debug/devbind-cli"

if [ ! -f "$DEVBIND" ]; then
    echo "❌ devbind-cli not found. Building it..."
    cargo build -p devbind-cli
fi

APPS_DIR="./apps"

if [ ! -d "$APPS_DIR" ]; then
    echo "❌ $APPS_DIR not found. Run ./install_apps.sh first!"
    exit 1
fi

echo "🚀 Testing installed apps in $APPS_DIR with devbind run"

function test_app_run() {
    local app_name=$1
    local run_cmd=("$@")
    # remove the first argument (app_name)
    run_cmd=("${run_cmd[@]:1}")

    echo ""
    echo "======================================"
    echo "Testing $app_name"
    echo "======================================"

    cd "$APPS_DIR/$app_name"

    # Check if there's an 'app' subfolder (e.g. for React CRA)
    if [ -d "app" ]; then
        cd "app"
    fi

    echo "=> Running 'devbind run $app_name ${run_cmd[*]}'..."

    # Run in background without eval (so literal $PORT gets passed)
    "$DEVBIND" run "$app_name" "${run_cmd[@]}" > devbind.log 2>&1 &
    local PID=$!

    # Wait for devbind to allocate a port and print it
    for i in {1..20}; do
        if grep -q "http://127.0.0.1:" devbind.log 2>/dev/null; then
            break
        fi
        sleep 0.5
    done

    # Extract port from log: "🔗  myapp.test → http://127.0.0.1:45321"
    PORT=$(grep -oP "(?<=http://127.0.0.1:)\d+" devbind.log || echo "")

    if [ -z "$PORT" ]; then
        echo "❌ Failed to extract port from devbind output. Log:"
        cat devbind.log
        kill -9 $PID 2>/dev/null || true
        cd - > /dev/null
        return 1
    fi

    echo "=> Background PID: $PID, Assigned Port: $PORT"
    echo "=> Testing connection to http://$app_name.test:$PORT..."

    # Give the app a few seconds to boot up and bind to the port
    local success=false
    for i in {1..20}; do
        if curl -s -m 2 "http://$app_name.test:$PORT" > /dev/null; then
            success=true
            break
        fi
        sleep 0.5
    done

    if [ "$success" = true ]; then
        echo "✅ Success! $app_name app responded."
    else
        echo "❌ Failed to connect to $app_name app."
        echo "--- Log Output ---"
        cat devbind.log
    fi

    echo "=> Cleaning up $app_name..."
    kill -INT $PID 2>/dev/null || true
    wait $PID 2>/dev/null || true

    if [ "$success" = true ]; then
        echo "✅ OK"
    else
        echo "❌ FAILED"
        cd - > /dev/null
        return 1
    fi
    cd - > /dev/null
}


# Loop through generated apps and test them
for APP_PATH in "$APPS_DIR"/*; do
    if [ ! -d "$APP_PATH" ]; then
        continue
    fi

    APP_NAME=$(basename "$APP_PATH")

    case "$APP_NAME" in
        "react")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --host
            ;;
        "vue_js")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --host
            ;;
        "angular")
            test_app_run "$APP_NAME" npm run start -- --port '$PORT' --host 0.0.0.0
            ;;
        "svelte")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --host
            ;;
        "next_js")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --hostname 0.0.0.0
            ;;
        "nuxt_js")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --host 0.0.0.0
            ;;
        "remix")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --host
            ;;
        "astro")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --host
            ;;
        "solid_js")
            test_app_run "$APP_NAME" pnpm run dev --port '$PORT' --host
            ;;
        "express_js" | "node_js" | "koa")
            test_app_run "$APP_NAME" node index.js
            ;;
        "django")
            test_app_run "$APP_NAME" python3 manage.py runserver '0.0.0.0:$PORT'
            ;;
        "flask")
            test_app_run "$APP_NAME" .venv/bin/python -m flask --app main run --host 0.0.0.0 --port '$PORT'
            ;;
        "fastapi")
            test_app_run "$APP_NAME" .venv/bin/python -m uvicorn main:app --host 0.0.0.0 --port '$PORT'
            ;;
        "ember_js")
            test_app_run "$APP_NAME" pnpm dlx http-server -p '$PORT'
            ;;
        "meteor_js")
            test_app_run "$APP_NAME" node index.js
            ;;
        "laravel")
            test_app_run "$APP_NAME" php artisan serve --host=0.0.0.0 --port='$PORT'
            ;;
        "ruby_on_rails")
            test_app_run "$APP_NAME" rails server -p '$PORT' -b 0.0.0.0
            ;;
        *)
            echo "⏭️ Skipping automated test for $APP_NAME (no test config defined)"
            ;;
    esac
done

echo ""
echo "🎉 Finished testing apps in $APPS_DIR/"
