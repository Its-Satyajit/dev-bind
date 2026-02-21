# DevBind Framework Guide

Different frameworks run their development servers differently. Below is a reference for using `devbind run` with all major web frameworks.

## Framework Support Matrix

| Framework | Command | Notes |
|---|---|---|
| **React** (Vite) | `devbind run react pnpm dev --port $PORT --host` | Vite needs `--host` for IPv4 binding |
| **Vue.js** (Vite) | `devbind run vuejs pnpm dev --port $PORT --host` | Vite needs `--host` for IPv4 binding |
| **Svelte** (Vite) | `devbind run svelte pnpm dev --port $PORT --host` | Vite needs `--host` for IPv4 binding |
| **Solid.js** (Vite) | `devbind run solidjs pnpm dev --port $PORT --host` | Vite needs `--host` for IPv4 binding |
| **Lit** (Vite) | `devbind run lit pnpm dev --port $PORT --host` | Vite needs `--host` for IPv4 binding |
| **Preact** (Vite) | `devbind run preact pnpm dev --port $PORT --host` | Vite needs `--host` for IPv4 binding |
| **Next.js** | `devbind run nextjs pnpm dev` | Works automatically |
| **Nuxt.js** | `devbind run nuxtjs pnpm dev` | Works automatically |
| **Angular** | `devbind run angular npm run ng serve --port $PORT --host 0.0.0.0` | Needs `--host 0.0.0.0` |
| **Astro** | `devbind run astro pnpm dev --port $PORT --host` | Vite-based, needs `--host` |
| **Remix** (React Router) | `devbind run remix pnpm dev --port $PORT` | Reads PORT from env |
| **NestJS** | `devbind run nestjs pnpm nest start --port $PORT` | Needs `--port` flag |
| **Express.js** | `devbind run express pnpm start` | Reads `$PORT` from env |
| **Koa** | `devbind run koa node index.js` | Reads `$PORT` from env |
| **Plain Node.js** | `devbind run nodejs node index.js` | Reads `$PORT` from env |
| **Ember.js** | `devbind run ember_js ember serve --port $PORT` | Using Ember CLI |
| **Meteor.js** | `devbind run meteor_js meteor run --port $PORT` | Using Meteor CLI |
| **Blazor** | `devbind run blazor dotnet run --urls http://0.0.0.0:$PORT` | .NET Core apps |
| **Django** | `devbind run django python manage.py runserver 0.0.0.0:$PORT` | Bind to all interfaces |
| **Flask** | `devbind run flask .venv/bin/python -m flask --app main run --host 0.0.0.0 --port $PORT` | Use venv python; `--app` names your entry file |
| **FastAPI** | `devbind run fastapi uvicorn main:app --host 0.0.0.0 --port $PORT` | Bind to all interfaces |
| **PHP** | `devbind run php php -S 0.0.0.0:$PORT` | PHP built-in server |

> **Note:** The `--host` flag is required for Vite-based frameworks because Vite defaults to IPv6-only (`::1`), while DevBind proxies to IPv4 `127.0.0.1`. Without `--host`, you will get a *Bad Gateway* error.

## Advanced Config by Framework

### Allowing Your `.test` Domain (Vite)

Vite 5+ validates the `Host` header by default. Add your domain to `vite.config.js/ts`:

```javascript
export default defineConfig({
  server: {
    allowedHosts: ['react.test', 'myapp.test'], // your devbind domain(s)
  }
})
```

### Allowing Your `.test` Domain (Angular)

Pass `--allowed-hosts` on the CLI or add to `angular.json`:
```bash
devbind run myapp npm run ng -- serve --port $PORT --host 0.0.0.0
```

> **Mobile Frameworks Note:** Frameworks like NativeScript, Apache Cordova, React Native, and Flutter are heavily focus on mobile architectures and typically do not run a standard local web server target that makes sense for DevBind to proxy. DevBind is primarily tailored to HTTP web servers and dev tools.
