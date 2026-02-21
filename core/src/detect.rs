//! Framework / dev-server auto-detection for `devbind run`.
//!
//! Call [`detect_command`] with the project's root directory.  It returns a
//! `Vec<String>` of the command + args to pass to `devbind run`, or `None`
//! when the directory cannot be identified as a known project type.
//!
//! Detection uses a **priority-ordered** rule list; the first match wins.
//! All returned `$PORT` tokens are literal – the caller substitutes the real
//! port before spawning.

use std::path::Path;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Read a file to a string; return empty string on any error.
fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Does the directory contain a file matching any of the given names?
fn has_file(dir: &Path, names: &[&str]) -> bool {
    names.iter().any(|n| dir.join(n).is_file())
}

/// Does `text` contain `needle` (case-insensitive)?
fn contains(text: &str, needle: &str) -> bool {
    text.to_lowercase().contains(&needle.to_lowercase())
}

/// Read `package.json` from `dir` (returns empty string if absent).
fn read_package_json(dir: &Path) -> String {
    read_file(&dir.join("package.json"))
}

/// Read `pyproject.toml` from `dir` (returns empty string if absent).
fn read_pyproject(dir: &Path) -> String {
    read_file(&dir.join("pyproject.toml"))
}

/// Detect which Node package manager is available (prefer pnpm, fall back to npm).
fn node_pm(dir: &Path) -> &'static str {
    if dir.join("pnpm-lock.yaml").is_file() || dir.join("pnpm-workspace.yaml").is_file() {
        "pnpm"
    } else {
        "npm"
    }
}

// ── public API ────────────────────────────────────────────────────────────────

/// Inspect `dir` and return the inferred dev-server command, or `None`.
///
/// Returned tokens may contain the literal string `$PORT` — it will be
/// substituted by the caller before the process is spawned.
pub fn detect_command(dir: &Path) -> Option<Vec<String>> {
    // ── 1. Next.js ───────────────────────────────────────────────────────────
    if has_file(
        dir,
        &[
            "next.config.js",
            "next.config.ts",
            "next.config.mjs",
            "next.config.cjs",
        ],
    ) {
        let pm = node_pm(dir);
        return Some(vec![
            pm.into(),
            "run".into(),
            "dev".into(),
            "--port".into(),
            "$PORT".into(),
            "--hostname".into(),
            "0.0.0.0".into(),
        ]);
    }

    // ── 2. NestJS (nest-cli.json marker) ─────────────────────────────────────
    if has_file(dir, &["nest-cli.json"]) {
        let pm = node_pm(dir);
        return Some(vec![pm.into(), "run".into(), "start:dev".into()]);
    }

    // ── 3. Nuxt ──────────────────────────────────────────────────────────────
    {
        let pkg = read_package_json(dir);
        if !pkg.is_empty() && (contains(&pkg, "\"nuxt\"") || contains(&pkg, "'nuxt'")) {
            let pm = node_pm(dir);
            return Some(vec![
                pm.into(),
                "run".into(),
                "dev".into(),
                "--port".into(),
                "$PORT".into(),
                "--host".into(),
                "0.0.0.0".into(),
            ]);
        }
    }

    // ── 4. Remix ─────────────────────────────────────────────────────────────
    {
        let pkg = read_package_json(dir);
        if !pkg.is_empty() && contains(&pkg, "@remix-run") {
            let pm = node_pm(dir);
            return Some(vec![
                pm.into(),
                "run".into(),
                "dev".into(),
                "--port".into(),
                "$PORT".into(),
                "--host".into(),
            ]);
        }
    }

    // ── 5. Astro ─────────────────────────────────────────────────────────────
    if has_file(
        dir,
        &["astro.config.js", "astro.config.ts", "astro.config.mjs"],
    ) {
        let pm = node_pm(dir);
        return Some(vec![
            pm.into(),
            "run".into(),
            "dev".into(),
            "--port".into(),
            "$PORT".into(),
            "--host".into(),
        ]);
    }

    // ── 6. Angular ───────────────────────────────────────────────────────────
    if has_file(dir, &["angular.json"]) {
        let pm = node_pm(dir);
        let run = if pm == "npm" { "npm" } else { pm };
        // angular ng serve doesn't support $PORT via pnpm run dev easily;
        // use `npm run start` with the extra flags.
        return Some(vec![
            run.into(),
            "run".into(),
            "start".into(),
            "--".into(),
            "--port".into(),
            "$PORT".into(),
            "--host".into(),
            "0.0.0.0".into(),
        ]);
    }

    // ── 7. Ember ─────────────────────────────────────────────────────────────
    if has_file(dir, &["ember-cli-build.js"]) {
        return Some(vec![
            "pnpm".into(),
            "dlx".into(),
            "http-server".into(),
            "-p".into(),
            "$PORT".into(),
        ]);
    }

    // ── 8. Vite (any framework via vite.config.*) ────────────────────────────
    if has_file(
        dir,
        &[
            "vite.config.js",
            "vite.config.ts",
            "vite.config.mjs",
            "vite.config.cjs",
        ],
    ) {
        let pm = node_pm(dir);
        return Some(vec![
            pm.into(),
            "run".into(),
            "dev".into(),
            "--port".into(),
            "$PORT".into(),
            "--host".into(),
        ]);
    }

    // ── 9. SvelteKit (svelte.config.*) ───────────────────────────────────────
    if has_file(dir, &["svelte.config.js", "svelte.config.ts"]) {
        let pm = node_pm(dir);
        return Some(vec![
            pm.into(),
            "run".into(),
            "dev".into(),
            "--port".into(),
            "$PORT".into(),
            "--host".into(),
        ]);
    }

    // ── 10. Generic package.json with "dev" script ───────────────────────────
    {
        let pkg = read_package_json(dir);
        if !pkg.is_empty() && contains(&pkg, "\"dev\"") {
            let pm = node_pm(dir);
            return Some(vec![
                pm.into(),
                "run".into(),
                "dev".into(),
                "--port".into(),
                "$PORT".into(),
                "--host".into(),
            ]);
        }
    }

    // ── 11. Django ───────────────────────────────────────────────────────────
    if has_file(dir, &["manage.py"]) {
        return Some(vec![
            "python3".into(),
            "manage.py".into(),
            "runserver".into(),
            "0.0.0.0:$PORT".into(),
        ]);
    }

    // ── 12. Flask ────────────────────────────────────────────────────────────
    {
        let pyproject = read_pyproject(dir);
        if !pyproject.is_empty() && contains(&pyproject, "flask") {
            // Prefer a venv if present
            let interpreter = if dir.join(".venv/bin/python").is_file() {
                ".venv/bin/python"
            } else {
                "python3"
            };
            // Try to find the app module name (main.py → "main", app.py → "app")
            let app_module = if dir.join("main.py").is_file() {
                "main"
            } else {
                "app"
            };
            return Some(vec![
                interpreter.into(),
                "-m".into(),
                "flask".into(),
                "--app".into(),
                app_module.into(),
                "run".into(),
                "--host".into(),
                "0.0.0.0".into(),
                "--port".into(),
                "$PORT".into(),
            ]);
        }
    }

    // ── 13. FastAPI / uvicorn ─────────────────────────────────────────────────
    {
        let pyproject = read_pyproject(dir);
        if !pyproject.is_empty()
            && (contains(&pyproject, "fastapi") || contains(&pyproject, "uvicorn"))
        {
            let interpreter = if dir.join(".venv/bin/python").is_file() {
                ".venv/bin/python"
            } else {
                "python3"
            };
            // Assume the FastAPI instance is `app` in `main.py`
            return Some(vec![
                interpreter.into(),
                "-m".into(),
                "uvicorn".into(),
                "main:app".into(),
                "--host".into(),
                "0.0.0.0".into(),
                "--port".into(),
                "$PORT".into(),
            ]);
        }
    }

    // ── 14. Laravel ──────────────────────────────────────────────────────────
    if has_file(dir, &["artisan"]) {
        return Some(vec![
            "php".into(),
            "artisan".into(),
            "serve".into(),
            "--host=0.0.0.0".into(),
            "--port=$PORT".into(),
        ]);
    }

    // ── 15. Ruby on Rails ────────────────────────────────────────────────────
    if has_file(dir, &["bin/rails"]) || dir.join("bin").join("rails").is_file() {
        return Some(vec![
            "rails".into(),
            "server".into(),
            "-p".into(),
            "$PORT".into(),
            "-b".into(),
            "0.0.0.0".into(),
        ]);
    }

    None
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmpdir() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn touch(dir: &Path, rel: &str) {
        let full = dir.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(full, "").expect("touch");
    }

    fn write(dir: &Path, rel: &str, contents: &str) {
        let full = dir.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(full, contents).expect("write");
    }

    // Helper: join args into a string for readable assertions
    fn joined(cmd: Option<Vec<String>>) -> Option<String> {
        cmd.map(|v| v.join(" "))
    }

    #[test]
    fn empty_dir_returns_none() {
        let dir = tmpdir();
        assert!(detect_command(dir.path()).is_none());
    }

    #[test]
    fn detects_nextjs_via_config() {
        let dir = tmpdir();
        touch(dir.path(), "next.config.ts");
        touch(dir.path(), "pnpm-lock.yaml");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("pnpm"), "expected pnpm, got: {cmd}");
        assert!(cmd.contains("dev"), "expected dev script, got: {cmd}");
        assert!(
            cmd.contains("--hostname"),
            "expected --hostname, got: {cmd}"
        );
        assert!(cmd.contains("$PORT"), "expected $PORT, got: {cmd}");
    }

    #[test]
    fn detects_nestjs_via_nest_cli_json() {
        let dir = tmpdir();
        touch(dir.path(), "nest-cli.json");
        touch(dir.path(), "pnpm-lock.yaml");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("start:dev"), "expected start:dev, got: {cmd}");
    }

    #[test]
    fn detects_nuxt_via_package_json() {
        let dir = tmpdir();
        write(
            dir.path(),
            "package.json",
            r#"{"dependencies":{"nuxt":"^3.0.0"}}"#,
        );
        touch(dir.path(), "pnpm-lock.yaml");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("dev"), "expected dev, got: {cmd}");
        assert!(cmd.contains("--host"), "expected --host, got: {cmd}");
    }

    #[test]
    fn detects_vite_via_config() {
        let dir = tmpdir();
        touch(dir.path(), "vite.config.ts");
        write(dir.path(), "package.json", r#"{"scripts":{"dev":"vite"}}"#);
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("dev"), "expected dev, got: {cmd}");
        assert!(cmd.contains("--host"), "expected --host, got: {cmd}");
    }

    #[test]
    fn detects_svelte_via_config() {
        let dir = tmpdir();
        touch(dir.path(), "svelte.config.js");
        touch(dir.path(), "pnpm-lock.yaml");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("dev"), "got: {cmd}");
        assert!(cmd.contains("--host"), "got: {cmd}");
    }

    #[test]
    fn detects_angular_via_angular_json() {
        let dir = tmpdir();
        touch(dir.path(), "angular.json");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("start"), "expected start script, got: {cmd}");
        assert!(cmd.contains("--port"), "got: {cmd}");
        assert!(cmd.contains("0.0.0.0"), "got: {cmd}");
    }

    #[test]
    fn detects_astro_via_config() {
        let dir = tmpdir();
        touch(dir.path(), "astro.config.mjs");
        touch(dir.path(), "pnpm-lock.yaml");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("dev"), "got: {cmd}");
        assert!(cmd.contains("--host"), "got: {cmd}");
    }

    #[test]
    fn detects_django_via_manage_py() {
        let dir = tmpdir();
        touch(dir.path(), "manage.py");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("manage.py"), "got: {cmd}");
        assert!(cmd.contains("runserver"), "got: {cmd}");
        assert!(cmd.contains("$PORT"), "got: {cmd}");
    }

    #[test]
    fn detects_flask_via_pyproject() {
        let dir = tmpdir();
        write(
            dir.path(),
            "pyproject.toml",
            "[project]\ndependencies = [\"flask>=3.0\"]",
        );
        touch(dir.path(), "main.py");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("flask"), "got: {cmd}");
        assert!(cmd.contains("--app main"), "got: {cmd}");
        assert!(cmd.contains("$PORT"), "got: {cmd}");
    }

    #[test]
    fn detects_fastapi_via_pyproject() {
        let dir = tmpdir();
        write(
            dir.path(),
            "pyproject.toml",
            "[project]\ndependencies = [\"fastapi\"]",
        );
        touch(dir.path(), "main.py");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("uvicorn"), "got: {cmd}");
        assert!(cmd.contains("main:app"), "got: {cmd}");
        assert!(cmd.contains("$PORT"), "got: {cmd}");
    }

    #[test]
    fn detects_laravel_via_artisan() {
        let dir = tmpdir();
        touch(dir.path(), "artisan");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("artisan"), "got: {cmd}");
        assert!(cmd.contains("serve"), "got: {cmd}");
    }

    #[test]
    fn detects_rails_via_bin_rails() {
        let dir = tmpdir();
        touch(dir.path(), "bin/rails");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("rails server"), "got: {cmd}");
        assert!(cmd.contains("$PORT"), "got: {cmd}");
    }

    #[test]
    fn generic_package_json_with_dev_script() {
        let dir = tmpdir();
        write(
            dir.path(),
            "package.json",
            r#"{"scripts":{"dev":"node server.js"}}"#,
        );
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("dev"), "got: {cmd}");
    }

    #[test]
    fn flask_detects_app_module_when_no_main_py() {
        let dir = tmpdir();
        write(
            dir.path(),
            "pyproject.toml",
            "[project]\ndependencies = [\"flask\"]",
        );
        // no main.py — should fall back to "app"
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains("--app app"), "got: {cmd}");
    }

    #[test]
    fn uses_venv_interpreter_when_present() {
        let dir = tmpdir();
        write(
            dir.path(),
            "pyproject.toml",
            "[project]\ndependencies = [\"flask\"]",
        );
        // Create a fake .venv/bin/python file
        touch(dir.path(), ".venv/bin/python");
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.contains(".venv/bin/python"), "got: {cmd}");
    }

    #[test]
    fn npm_used_when_no_pnpm_lockfile() {
        let dir = tmpdir();
        touch(dir.path(), "next.config.js");
        // No pnpm-lock.yaml → should use npm
        let cmd = joined(detect_command(dir.path())).unwrap();
        assert!(cmd.starts_with("npm"), "expected npm, got: {cmd}");
    }
}
