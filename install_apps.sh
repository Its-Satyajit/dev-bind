#!/bin/bash
set -e

# Support user-local pnpm installation
export PATH="$HOME/.npm-global/bin:$PATH"

# Ensure we're in the dev-bind repo root
cd "$(dirname "$0")"

APPS_DIR="./apps"
APPS_LIST="apps.json"

if [ ! -f "$APPS_LIST" ]; then
    echo "❌ $APPS_LIST not found!"
    exit 1
fi

mkdir -p "$APPS_DIR"

echo "🚀 Installing apps listed in $APPS_LIST into $APPS_DIR"

# Parse JSON array using jq (if installed), fallback to grep if not
if command -v jq &> /dev/null; then
    jq -r '.[]' "$APPS_LIST" > .apps_tmp
else
    # Simple fallback parser for the specific JSON format
    grep -oP '"\K[^"]+(?=")' "$APPS_LIST" > .apps_tmp
fi

while read -r APP_NAME; do
    # Skip empty lines
    [ -z "$APP_NAME" ] && continue

    echo "======================================"
    echo "📦 Installing: $APP_NAME"
    echo "======================================"

    # Create a safe directory name (lowercase, no spaces)
    DIR_NAME=$(echo "$APP_NAME" | tr '[:upper:]' '[:lower:]' | sed 's/[ .]/_/g')
    TARGET="$APPS_DIR/$DIR_NAME"

    if [ -d "$TARGET" ]; then
        echo "⏭️  $TARGET already exists. Skipping."
        continue
    fi

    mkdir -p "$TARGET"
    cd "$TARGET"

    # Detect the framework and run the specific initialization command
    (
        set +e
        case "$APP_NAME" in
            "React")
                if command -v pnpm &> /dev/null; then
                    pnpm create vite@latest app -- --template react || echo "Failed to init React"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Vue.js")
                if command -v pnpm &> /dev/null; then
                    pnpm create vite@latest app -- --template vue || echo "Failed to init Vue"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Angular")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx @angular/cli new app --defaults || echo "Failed to init Angular"
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Svelte")
                if command -v pnpm &> /dev/null; then
                    pnpm create vite@latest app -- --template svelte || echo "Failed to init Svelte"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Next.js")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx create-next-app@latest app --yes || echo "Failed to init Next.js"
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Nuxt.js")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx nuxi init app --template minimal --force || echo "Failed to init Nuxt"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Remix")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx create-remix@latest app --template remix --yes || echo "Failed to init Remix"
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Astro")
                if command -v pnpm &> /dev/null; then
                    pnpm create astro@latest app -- --template minimal || echo "Failed to init Astro"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Solid.js")
                if command -v pnpm &> /dev/null; then
                    pnpm create solid@latest app -- --template solid || echo "Failed to init Solid"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Ember.js")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx ember-cli new app --skip-npm || echo "Failed to init Ember"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Lit")
                if command -v pnpm &> /dev/null; then
                    pnpm create vite@latest app -- --template lit || echo "Failed to init Lit"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Preact")
                if command -v pnpm &> /dev/null; then
                    pnpm create vite@latest app -- --template preact || echo "Failed to init Preact"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Express.js")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx express-generator app --no-view || echo "Failed to init Express"
                    cd app && pnpm install
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Node.js" | "Koa")
                if command -v pnpm &> /dev/null; then
                    mkdir app && cd app && pnpm init
                    echo "const http = require('http'); http.createServer((q,s)=>s.end('Hello')).listen(process.env.PORT||3000);" > index.js
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Django")
                if command -v django-admin &> /dev/null; then
                    django-admin startproject app || echo "Failed to init Django"
                else echo "Skiping (django-admin missing)"; fi
                ;;
            "Flask" | "FastAPI")
                if command -v python3 &> /dev/null; then
                    mkdir app && cd app
                    python3 -m venv venv || echo "Failed to create venv"
                    echo "print('Hello World')" > main.py
                else echo "Skiping (python3 missing)"; fi
                ;;
            "Laravel")
                if command -v composer &> /dev/null; then
                    composer create-project laravel/laravel app --prefer-dist || echo "Failed to init Laravel"
                else echo "Skiping (composer missing)"; fi
                ;;
            "Ruby on Rails")
                if command -v rails &> /dev/null; then
                    rails new app --skip-test || echo "Failed to init Rails"
                else echo "Skiping (rails missing)"; fi
                ;;
            "Spring Boot")
                curl "https://start.spring.io/starter.zip?type=maven-project&language=java&groupId=com.example&artifactId=myapp" -o app.zip
                unzip app.zip -d app && rm app.zip
                ;;
            "ASP.NET Core" | "Blazor")
                if command -v dotnet &> /dev/null; then
                    dotnet new webapp -n app || echo "Failed to init .NET"
                else echo "Skiping (dotnet missing)"; fi
                ;;
            "NestJS")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx @nestjs/cli new app --skip-install || echo "Failed to init NestJS"
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Phoenix")
                if command -v mix &> /dev/null; then
                    mix phx.new app --no-ecto --no-install || echo "Failed to init Phoenix"
                else echo "Skipping (mix missing)"; fi
                ;;
            "Meteor.js")
                if command -v meteor &> /dev/null; then
                    meteor create app || echo "Failed to init Meteor"
                else echo "Skipping (meteor missing)"; fi
                ;;
            "Flutter")
                if command -v flutter &> /dev/null; then
                    flutter create app || echo "Failed to init Flutter"
                else echo "Skipping (flutter missing)"; fi
                ;;
            "React Native")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx create-expo-app app --template blank || echo "Failed to init Expo"
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "Quasar Framework")
                if command -v pnpm &> /dev/null; then
                    pnpm dlx @quasar/cli create app --branch next --yes || echo "Failed to init Quasar"
                else echo "Skipping (pnpm missing)"; fi
                ;;
            "NativeScript" | "Apache Cordova")
                echo "Mobile framework; creating README stub."
                mkdir app 2>/dev/null || true
                echo "# $APP_NAME" > app/README.md
                echo "Not HTTP-server compatible." >> app/README.md
                ;;
            *)
                echo "❓ Don't know how to automatically install '$APP_NAME' non-interactively. Creating empty directory."
                mkdir app 2>/dev/null || true
                touch app/README.md
                ;;
        esac
    )

    # Go back to repo root
    cd ../../
done < .apps_tmp

rm -f .apps_tmp

echo ""
echo "🎉 App scaffolding complete! Look in $APPS_DIR/"
