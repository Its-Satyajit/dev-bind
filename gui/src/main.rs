pub mod components;
pub mod service;
pub mod utils;

use components::daemon::DaemonTab;
use components::dns::DnsTab;
use components::help::HelpTab;
use components::mappings::MappingsTab;
use components::security::SecurityTab;
use devbind_core::config::DevBindConfig;
use dioxus::prelude::*;
use service::{is_service_active, is_service_installed};
use std::sync::{Arc, Mutex};
use utils::{get_config_path, is_proxy_running, which_devbind};

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
    let mut config = use_signal(|| DevBindConfig::load(&get_config_path()).unwrap_or_default());
    let new_domain = use_signal(|| String::new());
    let new_port = use_signal(|| String::new());
    let error_msg = use_signal(|| String::new());
    let success_msg = use_signal(|| String::new());
    let mut active_tab = use_signal(|| "dashboard");
    let mut dns_installed = use_signal(devbind_core::setup::is_dns_installed);

    // Proxy process handle (for manual start/stop, not the systemd path).
    let proxy_child: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
    let mut proxy_online = use_signal(is_proxy_running);
    let mut service_installed_sig = use_signal(is_service_installed);
    let mut service_active_sig = use_signal(is_service_active);
    let mut github_stars = use_signal(|| None::<u64>);

    use_future(move || async move {
        if let Ok(client) = reqwest::Client::builder().user_agent("devbind-gui").build() {
            if let Ok(res) = client
                .get("https://api.github.com/repos/Its-Satyajit/dev-bind")
                .send()
                .await
            {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let Some(stars) = json.get("stargazers_count").and_then(|v| v.as_u64()) {
                        github_stars.set(Some(stars));
                    }
                }
            }
        }
    });

    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            if let Ok(cfg) = DevBindConfig::load(&get_config_path()) {
                if *config.peek() != cfg {
                    config.set(cfg);
                }
            }
            if *proxy_online.peek() != is_proxy_running() {
                proxy_online.set(is_proxy_running());
            }
            if *service_active_sig.peek() != is_service_active() {
                service_active_sig.set(is_service_active());
            }
            if *service_installed_sig.peek() != is_service_installed() {
                service_installed_sig.set(is_service_installed());
            }
            if *dns_installed.peek() != devbind_core::setup::is_dns_installed() {
                dns_installed.set(devbind_core::setup::is_dns_installed());
            }
        }
    });

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
    let help_active = active_tab() == "help";

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
                    div {
                        class: "mt-2",
                        button {
                            class: "flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-white transition-colors focus:outline-none",
                            "data-tooltip": "View on GitHub",
                            onclick: move |_| { let _ = open::that("https://github.com/Its-Satyajit/dev-bind"); },
                            svg {
                                class: "w-3.5 h-3.5",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"
                                }
                            }
                            if let Some(stars) = github_stars() {
                                span { class: "mono", "{stars} ★" }
                            } else {
                                span { class: "mono", "GitHub" }
                            }
                        }
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
                            service_installed_sig.set(is_service_installed());
                            service_active_sig.set(is_service_active());
                            proxy_online.set(is_proxy_running());
                        },
                        "DAEMON"
                    }
                    button {
                        class: if help_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:text-[var(--text-main)] hover:bg-white/5 transition-all text-sm" },
                        "data-tooltip": "Get help and usage instructions",
                        onclick: move |_| active_tab.set("help"),
                        "HELP"
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
                            span { class: "text-green-500", "[OK] {success_msg()}" }
                        }
                        if !error_msg().is_empty() {
                            span { class: "text-red-500", "[ERROR] {error_msg()}" }
                        }
                    }
                }

                div { class: "p-10 flex-1 overflow-y-auto",
                    if dashboard_active {
                        MappingsTab {
                            config: config,
                            new_domain: new_domain,
                            new_port: new_port,
                            error_msg: error_msg,
                            success_msg: success_msg,
                        }
                    } else if dns_active {
                        DnsTab {
                            dns_installed: dns_installed,
                            error_msg: error_msg,
                            success_msg: success_msg,
                        }
                    } else if security_active {
                        SecurityTab {
                            error_msg: error_msg,
                            success_msg: success_msg,
                        }
                    } else if daemon_active {
                        DaemonTab {
                            proxy_online: proxy_online,
                            service_installed: service_installed_sig,
                            service_active: service_active_sig,
                            error_msg: error_msg,
                            success_msg: success_msg,
                        }
                    } else if help_active {
                        HelpTab {}
                    }
                }
            }
        }
    }
}
