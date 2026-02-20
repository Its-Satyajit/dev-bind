use devbind_core::config::DevBindConfig;
use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn get_config_path() -> PathBuf {
    let mut path = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        PathBuf::from(format!("/home/{}/.config", sudo_user))
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"))
    };
    path.push("devbind");
    path.push("config.toml");
    path
}

fn get_config_dir() -> PathBuf {
    let mut p = get_config_path();
    p.pop();
    p
}

/// Resolve the installed devbind binary path (prefers ~/.local/bin, falls back to PATH).
fn which_devbind() -> String {
    if let Some(p) = dirs::home_dir()
        .map(|h| h.join(".local/bin/devbind"))
        .filter(|p| p.exists())
    {
        return p.to_string_lossy().into_owned();
    }
    "devbind".to_string()
}

/// Check whether the devbind proxy is actually listening on port 443.
fn is_proxy_running() -> bool {
    std::net::TcpStream::connect("127.0.0.1:443").is_ok()
}

/// Check whether the systemd user service is active.
fn is_service_active() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "devbind"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check whether the systemd user service is installed (unit file exists).
fn is_service_installed() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".config/systemd/user/devbind.service").exists())
        .unwrap_or(false)
}

/// Write the systemd user service file and enable + start it.
fn install_service(devbind_bin: &str) -> Result<(), String> {
    let service_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".config/systemd/user");
    std::fs::create_dir_all(&service_dir).map_err(|e| e.to_string())?;

    let service_content = format!(
        "[Unit]\nDescription=DevBind Local Dev SSL Reverse Proxy\nAfter=network.target\n\n\
         [Service]\nExecStart={bin} start\nRestart=on-failure\nRestartSec=5\n\n\
         [Install]\nWantedBy=default.target\n",
        bin = devbind_bin
    );
    std::fs::write(service_dir.join("devbind.service"), service_content)
        .map_err(|e| e.to_string())?;

    for args in &[
        vec!["--user", "daemon-reload"],
        vec!["--user", "enable", "devbind"],
        vec!["--user", "start", "devbind"],
    ] {
        let status = std::process::Command::new("systemctl")
            .args(args)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("systemctl {} failed", args.join(" ")));
        }
    }
    Ok(())
}

/// Stop, disable and remove the systemd user service.
fn uninstall_service() -> Result<(), String> {
    for args in &[
        vec!["--user", "stop", "devbind"],
        vec!["--user", "disable", "devbind"],
    ] {
        let _ = std::process::Command::new("systemctl").args(args).status();
    }
    if let Some(path) = dirs::home_dir()
        .map(|h| h.join(".config/systemd/user/devbind.service"))
        .filter(|p| p.exists())
    {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    Ok(())
}

fn main() {
    tracing_subscriber::fmt::init();
    let cfg = dioxus::desktop::Config::default().with_window(
        dioxus::desktop::WindowBuilder::new()
            .with_title("DevBind")
            .with_inner_size(dioxus::desktop::LogicalSize::new(900.0, 590.0)),
    );
    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
    let config = use_signal(|| DevBindConfig::load(&get_config_path()).unwrap_or_default());
    let mut new_domain = use_signal(|| String::new());
    let mut new_port = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());
    let mut success_msg = use_signal(|| String::new());
    let mut active_tab = use_signal(|| "dashboard");
    let mut dns_installed = use_signal(devbind_core::setup::is_dns_installed);

    // Proxy process handle (for manual start/stop, not the systemd path).
    let proxy_child: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
    let mut proxy_online = use_signal(is_proxy_running);
    let mut service_installed = use_signal(is_service_installed);
    let mut service_active = use_signal(is_service_active);

    let update_config = move |cfg: DevBindConfig,
                              mut config_sig: Signal<DevBindConfig>,
                              mut err_sig: Signal<String>| {
        let path = get_config_path();
        if let Err(e) = cfg.save(&path) {
            err_sig.set(format!("Failed to save configuration: {}", e));
            return;
        }
        err_sig.set(String::new());
        config_sig.set(cfg);
    };

    let style_content = r#"
        @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
        @import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap');
        :root {
            --bg-main: #232629; --bg-sidebar: #31363b; --bg-card: #2a2e32;
            --text-main: #eff0f1; --text-muted: #7f8c8d;
            --accent: #3daee9; --accent-hover: #1d99f3;
            --border: #4d4d4d; --radius: 2px; --tooltip-bg: #1a1c1e;
        }
        @media (prefers-color-scheme: light) {
            :root {
                --bg-main: #eff0f1; --bg-sidebar: #fcfcfc; --bg-card: #ffffff;
                --text-main: #232629; --text-muted: #7f8c8d;
                --accent: #3daee9; --accent-hover: #1d99f3;
                --border: #cdd3da; --tooltip-bg: #232629;
            }
        }
        body { font-family: 'Inter', sans-serif; background-color: var(--bg-main); color: var(--text-main);
               transition: background-color 0.2s ease, color 0.2s ease; margin: 0; -webkit-font-smoothing: antialiased; }
        .mono { font-family: 'JetBrains Mono', monospace; }
        .sidebar { background-color: var(--bg-sidebar); border-right: 1px solid var(--border); }
        .terminal-block { background-color: rgba(0,0,0,0.05); border: 1px solid var(--border); border-radius: var(--radius); }
        .btn-action { background-color: var(--accent); color: white; border-radius: var(--radius); border: none; cursor: pointer; transition: all 0.2s ease; }
        .btn-action:hover { background-color: var(--accent-hover); }
        .btn-stop { background-color: #c0392b; color: white; border-radius: var(--radius); border: none; cursor: pointer; transition: all 0.2s ease; }
        .btn-stop:hover { background-color: #e74c3c; }
        input::placeholder, textarea::placeholder { color: var(--text-muted); opacity: 0.5; }
        textarea.terminal-input { background: rgba(0,0,0,0.2); color: var(--text-main); border: 1px solid var(--border);
            border-radius: var(--radius); font-family: 'JetBrains Mono', monospace; padding: 1rem; width: 100%; height: 300px; outline: none; resize: none; }
        .domain-link { transition: all 0.1s ease; cursor: pointer; }
        .domain-link:hover { text-decoration: underline; color: var(--accent); }
        [data-tooltip] { position: relative; }
        [data-tooltip]::after { content: attr(data-tooltip); position: absolute; bottom: 125%; left: 50%; transform: translateX(-50%);
            background-color: var(--tooltip-bg); color: #fff; padding: 4px 8px; border-radius: 4px; font-size: 10px;
            font-family: 'JetBrains Mono', monospace; white-space: nowrap; opacity: 0; visibility: hidden;
            transition: opacity 0.2s ease, transform 0.2s ease; z-index: 100; pointer-events: none;
            box-shadow: 0 4px 12px rgba(0,0,0,0.3); border: 1px solid var(--border); }
        [data-tooltip]:hover::after { opacity: 1; visibility: visible; transform: translateX(-50%) translateY(-4px); }
    "#;

    let dashboard_active = active_tab() == "dashboard";
    let security_active = active_tab() == "security";
    let dns_active = active_tab() == "dns";
    let daemon_active = active_tab() == "daemon";

    let proxy_child_start = proxy_child.clone();
    let proxy_child_stop = proxy_child.clone();

    rsx! {
        style { "{style_content}" }
        link { rel: "stylesheet", href: "https://unpkg.com/tailwindcss@2.2.19/dist/tailwind.min.css" }

        div { class: "flex h-screen overflow-hidden",

            // ── Sidebar ──────────────────────────────────────────────────────
            aside { class: "w-64 sidebar flex flex-col z-10",
                div { class: "p-8 mb-4",
                    h1 { class: "text-lg font-bold tracking-tighter mono flex items-center gap-2",
                        span { class: "text-[var(--accent)]", ">" }
                        "DevBind"
                    }
                }

                nav { class: "flex-1 space-y-px",
                    button {
                        class: if dashboard_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:text-[var(--text-main)] hover:bg-white/5 transition-all text-sm" },
                        "data-tooltip": "Manage proxy domains and local ports",
                        onclick: move |_| active_tab.set("dashboard"),
                        "MAPPINGS"
                    }
                    button {
                        class: if dns_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:text-[var(--text-main)] hover:bg-white/5 transition-all text-sm" },
                        "data-tooltip": "Manage DNS integration for .test domains",
                        onclick: move |_| {
                            active_tab.set("dns");
                            dns_installed.set(devbind_core::setup::is_dns_installed());
                        },
                        "DNS"
                    }
                    button {
                        class: if security_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:text-[var(--text-main)] hover:bg-white/5 transition-all text-sm" },
                        "data-tooltip": "Install or Revoke Root SSL Certificate trust",
                        onclick: move |_| active_tab.set("security"),
                        "SSL TRUST"
                    }
                    button {
                        class: if daemon_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:text-[var(--text-main)] hover:bg-white/5 transition-all text-sm" },
                        "data-tooltip": "Manage devbind as a systemd user service",
                        onclick: move |_| {
                            active_tab.set("daemon");
                            service_installed.set(is_service_installed());
                            service_active.set(is_service_active());
                            proxy_online.set(is_proxy_running());
                        },
                        "DAEMON"
                    }
                }

                // ── Proxy status + quick start/stop ──────────────────────────
                div { class: "p-6 border-t border-[var(--border)] space-y-3",
                    div { class: "flex items-center gap-2",
                        div {
                            class: if proxy_online() {
                                "w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.4)] animate-pulse"
                            } else {
                                "w-2 h-2 rounded-full bg-red-500/70"
                            }
                        }
                        span { class: "text-[10px] mono text-[var(--text-muted)]",
                            if proxy_online() { "PROXY_ONLINE" } else { "PROXY_OFFLINE" }
                        }
                    }

                    if proxy_online() {
                        button {
                            class: "btn-stop w-full py-2 text-[10px] mono font-bold",
                            "data-tooltip": "Stop the DevBind proxy (manual mode only)",
                            onclick: move |_| {
                                if let Ok(mut guard) = proxy_child_stop.lock() {
                                    if let Some(child) = guard.as_mut() {
                                        let _ = child.kill();
                                        let _ = child.wait();
                                    }
                                    *guard = None;
                                }
                                proxy_online.set(is_proxy_running());
                            },
                            "[ STOP PROXY ]"
                        }
                    } else {
                        button {
                            class: "btn-action w-full py-2 text-[10px] mono font-bold",
                            "data-tooltip": "Start the proxy manually (or install daemon for autostart)",
                            onclick: move |_| {
                                let bin = which_devbind();
                                if let Ok(child) = std::process::Command::new(&bin)
                                    .arg("start")
                                    .spawn()
                                {
                                    if let Ok(mut guard) = proxy_child_start.lock() {
                                        *guard = Some(child);
                                    }
                                    std::thread::sleep(std::time::Duration::from_millis(600));
                                    proxy_online.set(is_proxy_running());
                                }
                            },
                            "[ START PROXY ]"
                        }
                    }
                }
            }

            // ── Main content ─────────────────────────────────────────────────
            main { class: "flex-1 flex flex-col",

                header { class: "px-10 py-6 border-b border-[var(--border)] flex justify-between items-center",
                    h2 { class: "text-xs font-bold mono text-[var(--text-muted)]",
                        span { class: "text-[var(--accent)] mr-2", "~/" }
                        "{active_tab().to_uppercase()}"
                    }
                    div { class: "flex gap-4 mono text-[10px]",
                        if !success_msg().is_empty() {
                            span { class: "text-green-500", "✔ {success_msg()}" }
                        }
                        if !error_msg().is_empty() {
                            span { class: "text-red-500", "✘ {error_msg()}" }
                        }
                    }
                }

                div { class: "p-10 flex-1 overflow-y-auto",

                    // ── MAPPINGS tab ────────────────────────────────────────
                    if dashboard_active {
                        div { class: "max-w-4xl space-y-10",
                            div { class: "flex items-center gap-4 bg-[var(--bg-sidebar)] p-2 rounded border border-[var(--border)]",
                                span { class: "mono text-[var(--accent)] ml-4", "NEW>" }
                                input {
                                    class: "flex-1 bg-transparent border-none text-sm px-4 py-2 mono text-[var(--text-main)] outline-none",
                                    placeholder: "domain.test",
                                    "data-tooltip": "Enter the local domain name",
                                    value: "{new_domain()}",
                                    oninput: move |e| new_domain.set(e.value().clone())
                                }
                                span { class: "text-[var(--text-muted)] mono", ":" }
                                input {
                                    class: "w-24 bg-transparent border-none text-sm px-4 py-2 mono text-[var(--text-main)] outline-none text-center",
                                    placeholder: "3000",
                                    "data-tooltip": "Enter the backend service port",
                                    value: "{new_port()}",
                                    oninput: move |e| new_port.set(e.value().clone())
                                }
                                button {
                                    class: "btn-action px-6 py-2 text-xs font-bold mono",
                                    "data-tooltip": "Create or Update this mapping",
                                    onclick: move |_| {
                                        let mut cfg = config();
                                        if let Ok(p) = new_port().parse::<u16>() {
                                            let mut d = new_domain();
                                            if !d.is_empty() {
                                                if !d.ends_with(".test") { d.push_str(".test"); }
                                                if let Some(r) = cfg.routes.iter_mut().find(|r| r.domain == d) { r.port = p; }
                                                else { cfg.routes.push(devbind_core::config::RouteConfig { domain: d, port: p }); }
                                                update_config(cfg, config, error_msg);
                                                new_domain.set(String::new());
                                                new_port.set(String::new());
                                                success_msg.set("SAVED".to_string());
                                            }
                                        } else { error_msg.set("INVALID_PORT".to_string()); }
                                    },
                                    "SAVE ROUTE"
                                }
                            }

                            div { class: "terminal-block overflow-hidden",
                                div { class: "bg-black/5 px-8 py-3 border-b border-[var(--border)] flex justify-between items-center",
                                    span { class: "mono text-[9px] font-bold text-[var(--text-muted)]", "ACTIVE_MAPPINGS" }
                                    span { class: "mono text-[9px] text-[var(--text-muted)]", "COUNT: {config().routes.len()}" }
                                }
                                div { class: "p-4",
                                    if config().routes.is_empty() {
                                        p { class: "mono text-xs text-[var(--text-muted)] p-4", "# No mappings defined." }
                                    } else {
                                        table { class: "w-full mono text-xs",
                                            tbody {
                                                for r in config().routes {
                                                    tr { class: "hover:bg-black/5 group transition-colors",
                                                        key: "{r.domain}",
                                                        td { class: "px-4 py-3",
                                                            span { class: "text-[var(--accent)] mr-2", ">" }
                                                            span {
                                                                class: "domain-link font-medium",
                                                                "data-tooltip": "Click to open in default browser",
                                                                onclick: {
                                                                    let domain = r.domain.clone();
                                                                    move |_| { let _ = open::that(format!("https://{}", domain)); }
                                                                },
                                                                "{r.domain}"
                                                            }
                                                        }
                                                        td { class: "px-4 py-3 text-[var(--text-muted)]", "localhost:{r.port}" }
                                                        td { class: "px-4 py-3 text-right",
                                                            button {
                                                                class: "text-red-500 hover:text-red-600 transition-all font-bold px-2",
                                                                "data-tooltip": "Remove this mapping",
                                                                onclick: {
                                                                    let domain = r.domain.clone();
                                                                    move |_| {
                                                                        let mut cfg = config();
                                                                        cfg.routes.retain(|x| x.domain != domain);
                                                                        update_config(cfg, config, error_msg);
                                                                        success_msg.set("DELETED".to_string());
                                                                    }
                                                                },
                                                                "[ DELETE ]"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                    // ── DNS tab ────────────────────────────────────────────
                    } else if dns_active {
                        div { class: "max-w-2xl space-y-8",
                            div { class: "terminal-block p-8 space-y-6",
                                h3 { class: "mono text-sm font-bold flex items-center gap-3",
                                    span { class: "text-[var(--accent)]", "#" }
                                    "DNS_INTEGRATION"
                                }
                                p { class: "mono text-xs text-[var(--text-muted)] leading-relaxed",
                                    "Configure systemd-resolved to route all .test domains to DevBind's embedded DNS server. This eliminates the need for /etc/hosts editing."
                                }

                                div { class: "space-y-2 py-2",
                                    div { class: "flex items-center gap-3 mono text-xs",
                                        div {
                                            class: if dns_installed() { "w-2 h-2 rounded-full bg-green-500" } else { "w-2 h-2 rounded-full bg-red-500/70" }
                                        }
                                        span { class: "text-[var(--text-muted)]",
                                            if dns_installed() { "DNS_INSTALLED" } else { "DNS_NOT_INSTALLED" }
                                        }
                                    }
                                }

                                div { class: "mono text-[10px] text-[var(--text-muted)] bg-black/10 px-4 py-2 rounded",
                                    "NetworkManager dummy interface devbind0"
                                }

                                div { class: "flex gap-4 pt-4",
                                    if !dns_installed() {
                                        button {
                                            class: "btn-action px-8 py-3 mono text-xs font-bold",
                                            "data-tooltip": "Install DNS drop-in for .test domain resolution (elevated)",
                                            onclick: move |_| {
                                                match devbind_core::setup::install_dns(devbind_core::dns::DNS_LISTEN_ADDR) {
                                                    Ok(_) => { success_msg.set("DNS_INSTALLED".to_string()); error_msg.set(String::new()); },
                                                    Err(e) => { error_msg.set(format!("FAIL: {}", e)); success_msg.set(String::new()); }
                                                }
                                                dns_installed.set(devbind_core::setup::is_dns_installed());
                                            },
                                            "INSTALL DNS"
                                        }
                                    } else {
                                        button {
                                            class: "border border-red-500/20 text-red-500/60 px-8 py-3 mono text-xs font-bold rounded",
                                            "data-tooltip": "Remove DNS integration and stop .test resolution",
                                            onclick: move |_| {
                                                match devbind_core::setup::uninstall_dns() {
                                                    Ok(_) => { success_msg.set("DNS_REMOVED".to_string()); error_msg.set(String::new()); },
                                                    Err(e) => { error_msg.set(format!("FAIL: {}", e)); success_msg.set(String::new()); }
                                                }
                                                dns_installed.set(devbind_core::setup::is_dns_installed());
                                            },
                                            "UNINSTALL DNS"
                                        }
                                    }
                                }
                            }
                            p { class: "mono text-[10px] text-amber-500/50 px-4", "# Requires NetworkManager & systemd-resolved. The DNS server runs on port 53 when DevBind is active." }
                        }

                    // ── SSL TRUST tab ─────────────────────────────────────────
                    } else if security_active {
                        div { class: "max-w-2xl space-y-10",
                            div { class: "terminal-block p-8 space-y-6",
                                h3 { class: "mono text-sm font-bold flex items-center gap-3",
                                    span { class: "text-[var(--accent)]", "#" }
                                    "ROOT_CA_SETTINGS"
                                }
                                p { class: "mono text-xs text-[var(--text-muted)] leading-relaxed",
                                    "Manage system-wide SSL trust for your local .test domains. Installing the CA requires administrative access via system security prompt."
                                }
                                div { class: "flex gap-4 pt-4",
                                    button {
                                        class: "btn-action px-8 py-3 mono text-xs font-bold",
                                        "data-tooltip": "Install and trust DevBind Root CA (elevated)",
                                        onclick: move |_| {
                                            let dir = get_config_dir();
                                            match devbind_core::trust::install_root_ca(&dir) {
                                                Ok(_) => { success_msg.set("CA_TRUSTED".to_string()); error_msg.set(String::new()); },
                                                Err(e) => { error_msg.set(format!("FAIL: {}", e)); success_msg.set(String::new()); }
                                            }
                                        },
                                        "INSTALL TRUST"
                                    }
                                    button {
                                        class: "border border-red-500/20 text-red-500/60 px-8 py-3 mono text-xs font-bold rounded",
                                        "data-tooltip": "Remove DevBind Root CA from system trust store",
                                        onclick: move |_| {
                                            match devbind_core::trust::uninstall_root_ca() {
                                                Ok(_) => { success_msg.set("CA_REVOKED".to_string()); error_msg.set(String::new()); },
                                                Err(e) => { error_msg.set(format!("FAIL: {}", e)); success_msg.set(String::new()); }
                                            }
                                        },
                                        "REVOKE TRUST"
                                    }
                                }
                            }
                        }

                    // ── DAEMON tab ────────────────────────────────────────────
                    } else if daemon_active {
                        div { class: "max-w-2xl space-y-8",

                            // Status card
                            div { class: "terminal-block p-8 space-y-4",
                                h3 { class: "mono text-sm font-bold flex items-center gap-3",
                                    span { class: "text-[var(--accent)]", "#" }
                                    "SYSTEMD_USER_SERVICE"
                                }
                                p { class: "mono text-xs text-[var(--text-muted)] leading-relaxed",
                                    "Install devbind as a systemd user service so it starts automatically on login — no need to run 'devbind start' manually."
                                }

                                // Status rows
                                div { class: "space-y-2 py-2",
                                    div { class: "flex items-center gap-3 mono text-xs",
                                        div {
                                            class: if service_installed() { "w-2 h-2 rounded-full bg-green-500" } else { "w-2 h-2 rounded-full bg-red-500/70" }
                                        }
                                        span { class: "text-[var(--text-muted)]",
                                            if service_installed() { "SERVICE_INSTALLED" } else { "SERVICE_NOT_INSTALLED" }
                                        }
                                    }
                                    div { class: "flex items-center gap-3 mono text-xs",
                                        div {
                                            class: if service_active() { "w-2 h-2 rounded-full bg-green-500 animate-pulse" } else { "w-2 h-2 rounded-full bg-gray-500/50" }
                                        }
                                        span { class: "text-[var(--text-muted)]",
                                            if service_active() { "SERVICE_ACTIVE" } else { "SERVICE_INACTIVE" }
                                        }
                                    }
                                }

                                // Service file path info
                                div { class: "mono text-[10px] text-[var(--text-muted)] bg-black/10 px-4 py-2 rounded",
                                    "~/.config/systemd/user/devbind.service"
                                }

                                // Action buttons
                                div { class: "flex flex-wrap gap-3 pt-2",
                                    if !service_installed() {
                                        button {
                                            class: "btn-action px-6 py-2 mono text-xs font-bold",
                                            "data-tooltip": "Create and enable the systemd user service",
                                            onclick: move |_| {
                                                let bin = which_devbind();
                                                match install_service(&bin) {
                                                    Ok(_) => {
                                                        success_msg.set("DAEMON_INSTALLED".to_string());
                                                        error_msg.set(String::new());
                                                    }
                                                    Err(e) => {
                                                        error_msg.set(format!("DAEMON_ERROR: {}", e));
                                                        success_msg.set(String::new());
                                                    }
                                                }
                                                service_installed.set(is_service_installed());
                                                service_active.set(is_service_active());
                                                proxy_online.set(is_proxy_running());
                                            },
                                            "[ INSTALL DAEMON ]"
                                        }
                                    } else {
                                        if service_active() {
                                            button {
                                                class: "btn-stop px-6 py-2 mono text-xs font-bold",
                                                "data-tooltip": "Stop the systemd service",
                                                onclick: move |_| {
                                                    let _ = std::process::Command::new("systemctl")
                                                        .args(["--user", "stop", "devbind"])
                                                        .status();
                                                    service_active.set(is_service_active());
                                                    proxy_online.set(is_proxy_running());
                                                    success_msg.set("DAEMON_STOPPED".to_string());
                                                },
                                                "[ STOP SERVICE ]"
                                            }
                                        } else {
                                            button {
                                                class: "btn-action px-6 py-2 mono text-xs font-bold",
                                                "data-tooltip": "Start the systemd service",
                                                onclick: move |_| {
                                                    let _ = std::process::Command::new("systemctl")
                                                        .args(["--user", "start", "devbind"])
                                                        .status();
                                                    std::thread::sleep(std::time::Duration::from_millis(600));
                                                    service_active.set(is_service_active());
                                                    proxy_online.set(is_proxy_running());
                                                    success_msg.set("DAEMON_STARTED".to_string());
                                                },
                                                "[ START SERVICE ]"
                                            }
                                        }
                                        button {
                                            class: "border border-red-500/20 text-red-500/60 px-6 py-2 mono text-xs font-bold rounded",
                                            "data-tooltip": "Stop, disable and remove the service unit file",
                                            onclick: move |_| {
                                                match uninstall_service() {
                                                    Ok(_) => {
                                                        success_msg.set("DAEMON_REMOVED".to_string());
                                                        error_msg.set(String::new());
                                                    }
                                                    Err(e) => {
                                                        error_msg.set(format!("REMOVE_ERROR: {}", e));
                                                        success_msg.set(String::new());
                                                    }
                                                }
                                                service_installed.set(is_service_installed());
                                                service_active.set(is_service_active());
                                                proxy_online.set(is_proxy_running());
                                            },
                                            "[ UNINSTALL DAEMON ]"
                                        }
                                    }
                                }
                            }

                            p { class: "mono text-[10px] text-amber-500/50 px-4",
                                "# Uses 'systemctl --user' — no root required. Runs under your user session."
                            }
                        }
                    }
                }
            }
        }
    }
}
