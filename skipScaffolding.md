
# Quick rules of thumb (use these often)

* `--yes`, `-y`, `--default`, `--no-interactive`, or `--skip-prompts` — try these first.
* Many modern templates use the `npm create <tool>@latest <name> -- <flags>` pattern — the `--` passes flags to the create script.
* Use `--template <name>` or `--preset <name>` to pick a non-interactive preset.
* If a framework has no interactive CLI, use the zero-config starter (cookiecutter, Vite, boilerplate repo, or `startproject` style commands).

---

# Frontend / UI frameworks

* **React (via Vite)**
  `npm create vite@latest my-app -- --template react`
  (No prompts if template provided; you can add `-- --template react-ts` for TypeScript.)

* **Vue.js (create-vue)**
  `npm create vue@latest my-app -- --default`
  or explicitly: `npm create vue@latest my-app -- --no-typescript --no-router --no-pinia --no-vitest --no-eslint --no-prettier`

* **Angular (Angular CLI)**
  `ng new my-app --defaults`
  or `ng new my-app --skip-install --skip-tests` (use `--defaults` to accept default answers).

* **Svelte (SvelteKit)**
  `npm create svelte@latest my-app -- --template default`
  (Pick template on command; providing the template makes it non-interactive.)

* **Next.js**
  `npx create-next-app@latest my-app --yes`
  or `npm init next-app@latest my-app -- --typescript` (use `--yes` to accept defaults).

* **Nuxt.js**
  `npx nuxi init my-app` then `cd my-app && npm install`
  (`nuxi init` is typically non-interactive; for `create-nuxt-app` use `--no`/`--defaults` if available.)

* **Remix**
  `npx create-remix@latest --template remix` plus flags (many templates accept `--yes` or `--no-typescript` to skip prompts).

* **Astro**
  `npm create astro@latest my-site -- --template minimal`
  (Providing a template avoids the interactive prompt.)

* **Solid.js (SolidStart / Vite)**
  `npm create solid@latest my-app -- --template solid`
  (Use template param to skip prompts.)

* **Ember.js**
  `npx ember-cli new my-app --y`
  or `ember new my-app --skip-npm` (Ember CLI supports flags to avoid prompts.)

* **Backbone.js**
  No official interactive scaffold by default — typically add Backbone to a static project or use a Yeoman generator (if using a generator, use `--yes` or pass all options).

* **Alpine.js**
  No scaffolding CLI — just add the script tag or npm package (nothing to skip).

* **Lit**
  No mandated interactive generator; use `npm init @open-wc` or Vite templates (`-- --template lit`).

* **Preact**
  `npm create vite@latest my-app -- --template preact`
  (Using the template avoids prompts.)

---

# Backend / Full-stack frameworks

* **Express.js**
  `npx express-generator my-app --no-view` or `npx express-generator my-app -e`
  (express-generator is non-interactive when flags given.)

* **Node.js** (plain)
  `npm init -y` creates a package.json without prompts.

* **Django**
  `django-admin startproject myproject` — this is non-interactive by design.

* **Flask**
  No official scaffold; create files manually or use a cookiecutter: `cookiecutter gh:cookiecutter-flask/cookiecutter-flask` (cookiecutter supports `--no-input` to skip prompts).

* **FastAPI**
  No single official interactive generator; use a cookiecutter or template repo. For cookiecutter: `cookiecutter gh:tiangolo/full-stack-fastapi-postgresql --no-input`.

* **Laravel**
  `composer create-project laravel/laravel my-app --prefer-dist` (non-interactive).
  The Laravel installer `laravel new my-app` is non-interactive.

* **Ruby on Rails**
  `rails new my_app` — add flags like `--skip-test` or `--skip-bundle` to avoid extra steps. Rails `new` is non-interactive.

* **Spring Boot (Spring Initializr)**
  Use `curl` or `http` query to the start.spring.io API to get a zip:
  `curl "https://start.spring.io/starter.zip?type=maven-project&language=java&groupId=com.example&artifactId=myapp" -o myapp.zip` (completely non-interactive).

* **ASP.NET Core**
  `dotnet new webapp -n MyApp` — non-interactive.

* **Koa**
  No universal interactive scaffold; usually manual or use a generator (pass `--yes` or explicit flags).

* **NestJS**
  `nest new my-app --skip-install` or use `--package-manager npm` and pass options; `nest new` has `--yes` in some versions.

* **Phoenix (Elixir)**
  `mix phx.new my_app --no-ecto` or `--no-install` — flags skip prompts.

* **Meteor.js**
  `meteor create myapp` — non-interactive.

* **MERN Stack**
  There’s no single official MERN CLI; each piece (Mongo, Express, React, Node) has its own generator. Use templates/boilerplates (clone a repo) to avoid prompts.

* **Blazor**
  `dotnet new blazorserver -n MyBlazorApp` — non-interactive.

---

# Mobile / Cross-platform

* **Flutter**
  `flutter create my_app` — non-interactive, creates default project.

* **React Native (with Expo)**
  `npx create-expo-app my-app --template blank` (supplying template avoids prompts). For bare React Native, `npx react-native init MyApp` is non-interactive.

* **SwiftUI (Xcode)**
  Xcode projects are created in the GUI; for CLI you can script `xcodeproj` generation or use templates — usually not interactive.

* **Jetpack Compose**
  Use Android Studio templates or create Gradle project manually; no universal interactive CLI.

* **Kotlin Multiplatform**
  `gradle init` or templates; usually boilerplate repos are used.

* **Unity**
  Project creation is via the Unity Hub (GUI) or `unity` CLI in batchmode; you can create a project non-interactively with Unity Hub CLI.

* **.NET MAUI / Xamarin**
  `dotnet new maui -n MyMauiApp` — non-interactive.

* **Ionic**
  `ionic start my-app blank --type=angular --no-deps` or `--skip-install` — providing template skips prompts.

* **NativeScript**
  `ns create my-app --template @nativescript/template-blank` — choose template to skip prompts.

* **Apache Cordova**
  `cordova create myApp` — non-interactive; add platforms/plugins with flags.

* **Quasar Framework**
  `npx @quasar/cli create my-app --branch next --yes` or `quasar create` with `--yes`.

* **Sencha Ext JS**
  Usually enterprise tooling; the Sencha Cmd can scaffold apps with flags to be non-interactive.

---

# Generic patterns when a single exact flag isn’t available

1. **Provide a template/preset** with `--template`, `--preset`, or `--starter` so the tool has no questions.
2. **Use a non-interactive tool**: `cookiecutter --no-input`, `npm init -y`, `django-admin startproject`, `flutter create`.
3. **Script the setup**: clone a boilerplate repo and run `npm install`/`composer install` — no prompts.
4. **Pass `--yes`, `--defaults`, or `--no-interactive`** when supported.
5. **If CLI truly is interactive and has no flags, use `yes '' | <command>`** as a last resort (horrible but works for simple y/n prompts).

---

# Short, copyable cheat-sheet (pick one for your tool)

* `npm create vue@latest my-app -- --default`
* `npx create-next-app@latest my-app --yes`
* `npm create vite@latest my-app -- --template react`
* `ng new my-app --defaults`
* `django-admin startproject myproject`
* `composer create-project laravel/laravel my-app --prefer-dist`
* `rails new my_app`
* `flutter create my_app`
* `npx create-expo-app my-app --template blank`
* `cookiecutter gh:cookiecutter-flask/cookiecutter-flask --no-input`
* `curl "https://start.spring.io/starter.zip?..." -o myapp.zip`
