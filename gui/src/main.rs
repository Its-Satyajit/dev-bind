use devbind_core::config::DevBindConfig;
use devbind_core::hosts::HostsManager;
use dioxus::prelude::*;
use std::path::PathBuf;

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
    let mut hosts_content = use_signal(|| String::new());

    let update_config = move |cfg: DevBindConfig,
                              mut config_sig: Signal<DevBindConfig>,
                              mut err_sig: Signal<String>| {
        let path = get_config_path();
        if let Err(e) = cfg.save(&path) {
            err_sig.set(format!("Failed to save configuration: {}", e));
            return;
        }

        let hosts_path = PathBuf::from("/etc/hosts");
        let manager = HostsManager::new(&hosts_path);
        let domains: Vec<String> = cfg.routes.iter().map(|r| r.domain.clone()).collect();

        if let Err(e) = manager.update_routes(&domains) {
            err_sig.set(format!("Host configuration warning: {}", e));
        } else {
            err_sig.set(String::new());
        }
        config_sig.set(cfg);
    };

    let style_content = r#"
        @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
        @import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap');

        :root {
            --bg-main: #232629;
            --bg-sidebar: #31363b;
            --bg-card: #2a2e32;
            --text-main: #eff0f1;
            --text-muted: #7f8c8d;
            --accent: #3daee9;
            --accent-hover: #1d99f3;
            --border: #4d4d4d;
            --radius: 2px;
            --tooltip-bg: #1a1c1e;
        }

        @media (prefers-color-scheme: light) {
            :root {
                --bg-main: #eff0f1;
                --bg-sidebar: #fcfcfc;
                --bg-card: #ffffff;
                --text-main: #232629;
                --text-muted: #7f8c8d;
                --accent: #3daee9;
                --accent-hover: #1d99f3;
                --border: #cdd3da;
                --tooltip-bg: #232629;
            }
        }

        body {
            font-family: 'Inter', sans-serif; background-color: var(--bg-main); color: var(--text-main);
            transition: background-color 0.2s ease, color 0.2s ease; margin: 0;
            -webkit-font-smoothing: antialiased;
        }
        .mono { font-family: 'JetBrains Mono', monospace; }
        .sidebar { background-color: var(--bg-sidebar); border-right: 1px solid var(--border); }
        .terminal-block {
            background-color: rgba(0, 0, 0, 0.05); border: 1px solid var(--border); border-radius: var(--radius);
        }
        .btn-action {
            background-color: var(--accent); color: white; border-radius: var(--radius); border: none; cursor: pointer;
            transition: all 0.2s ease;
        }
        .btn-action:hover { background-color: var(--accent-hover); }
        input::placeholder, textarea::placeholder { color: var(--text-muted); opacity: 0.5; }
        textarea.terminal-input {
            background: rgba(0, 0, 0, 0.2);
            color: var(--text-main);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            font-family: 'JetBrains Mono', monospace;
            padding: 1rem;
            width: 100%;
            height: 300px;
            outline: none;
            resize: none;
        }
        .domain-link {
            transition: all 0.1s ease;
            cursor: pointer;
        }
        .domain-link:hover {
            text-decoration: underline;
            color: var(--accent);
        }

        /* Custom Tooltip Styles */
        [data-tooltip] {
            position: relative;
        }
        [data-tooltip]::after {
            content: attr(data-tooltip);
            position: absolute;
            bottom: 125%;
            left: 50%;
            transform: translateX(-50%);
            background-color: var(--tooltip-bg);
            color: #fff;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 10px;
            font-family: 'JetBrains Mono', monospace;
            white-space: nowrap;
            opacity: 0;
            visibility: hidden;
            transition: opacity 0.2s ease, transform 0.2s ease;
            z-index: 100;
            pointer-events: none;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
            border: 1px solid var(--border);
        }
        [data-tooltip]:hover::after {
            opacity: 1;
            visibility: visible;
            transform: translateX(-50%) translateY(-4px);
        }
    "#;

    let dashboard_active = active_tab() == "dashboard";
    let security_active = active_tab() == "security";
    let hosts_active = active_tab() == "hosts";

    rsx! {
        style { "{style_content}" }
        link { rel: "stylesheet", href: "https://unpkg.com/tailwindcss@2.2.19/dist/tailwind.min.css" }

        div { class: "flex h-screen overflow-hidden",

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
                        class: if hosts_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:text-[var(--text-main)] hover:bg-white/5 transition-all text-sm" },
                        "data-tooltip": "Edit system /etc/hosts file directly",
                        onclick: move |_| {
                            active_tab.set("hosts");
                            if let Ok(content) = std::fs::read_to_string("/etc/hosts") {
                                hosts_content.set(content);
                            }
                        },
                        "HOSTS FILE"
                    }
                    button {
                        class: if security_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:text-[var(--text-main)] hover:bg-white/5 transition-all text-sm" },
                        "data-tooltip": "Install or Revoke Root SSL Certificate trust",
                        onclick: move |_| active_tab.set("security"),
                        "SSL TRUST"
                    }
                }

                div { class: "p-8 border-t border-[var(--border)]",
                    div { class: "mono text-[9px] text-[var(--text-muted)] mb-1", "SYSTEM_SYNC: ACTIVE" }
                    div { class: "flex items-center gap-2",
                        div { class: "w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.4)] animate-pulse" }
                        span { class: "text-[10px] mono text-[var(--text-muted)]", "PROXY_ONLINE" }
                    }
                }
            }

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
                    if dashboard_active {
                        div { class: "max-w-4xl space-y-10",
                            div { class: "flex items-center gap-4 bg-[var(--bg-sidebar)] p-2 rounded border border-[var(--border)]",
                                span { class: "mono text-[var(--accent)] ml-4", "NEW>" }
                                input {
                                    class: "flex-1 bg-transparent border-none text-sm px-4 py-2 mono text-[var(--text-main)] outline-none",
                                    placeholder: "domain.local",
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
                                                if !d.ends_with(".local") { d.push_str(".local"); }
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
                                                                    move |_| {
                                                                        let _ = open::that(format!("https://{}", domain));
                                                                    }
                                                                },
                                                                "{r.domain}"
                                                            }
                                                        }
                                                        td { class: "px-4 py-3 text-[var(--text-muted)]", "localhost:{r.port}" }
                                                        td { class: "px-4 py-3 text-right",
                                                            button {
                                                                class: "text-red-500 hover:text-red-600 transition-all font-bold px-2",
                                                                "data-tooltip": "Remove this mapping and remove from hosts file",
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
                    } else if hosts_active {
                        div { class: "max-w-4xl space-y-6",
                            div { class: "terminal-block p-8 space-y-4",
                                h3 { class: "mono text-sm font-bold flex items-center gap-3",
                                    span { class: "text-[var(--accent)]", "#" }
                                    "HOSTS_FILE_EDITOR"
                                }
                                textarea {
                                    class: "terminal-input",
                                    value: "{hosts_content()}",
                                    oninput: move |e| hosts_content.set(e.value().clone())
                                }
                                div { class: "flex gap-4 pt-2",
                                    button {
                                        class: "btn-action px-8 py-3 mono text-xs font-bold",
                                        "data-tooltip": "Save manual changes to /etc/hosts (requires sudo)",
                                        onclick: move |_| {
                                            let content = hosts_content();
                                            let final_content = content.clone();
                                            match std::fs::write("/tmp/hosts_new", &final_content) {
                                                Ok(_) => {
                                                    let status = std::process::Command::new("pkexec")
                                                        .arg("cp")
                                                        .arg("/tmp/hosts_new")
                                                        .arg("/etc/hosts")
                                                        .status();
                                                    if let Ok(s) = status {
                                                        if s.success() {
                                                            success_msg.set("HOSTS_SAVED".to_string());
                                                            error_msg.set(String::new());
                                                        } else {
                                                            error_msg.set("ELEVATION_FAILED".to_string());
                                                        }
                                                    }
                                                },
                                                Err(e) => error_msg.set(format!("WRITE_FAIL: {}", e))
                                            }
                                        },
                                        "SAVE CHANGES"
                                    }
                                    button {
                                        class: "border border-red-500/20 text-red-500/60 px-8 py-3 mono text-xs font-bold rounded",
                                        "data-tooltip": "Reset /etc/hosts by removing all DevBind entries",
                                        onclick: move |_| {
                                            let manager = HostsManager::new(std::path::Path::new("/etc/hosts"));
                                            match manager.update_routes(&[]) {
                                                Ok(_) => {
                                                    success_msg.set("RESTORED_DEFAULTS".to_string());
                                                    error_msg.set(String::new());
                                                    if let Ok(content) = std::fs::read_to_string("/etc/hosts") {
                                                        hosts_content.set(content);
                                                    }
                                                },
                                                Err(e) => error_msg.set(format!("RESTORE_FAIL: {}", e))
                                            }
                                        },
                                        "RESTORE DEFAULTS"
                                    }
                                }
                            }
                            p { class: "mono text-[10px] text-amber-500/50 px-4", "# NOTE: Editing system files requires administrative authorization." }
                        }
                    } else if security_active {
                        div { class: "max-w-2xl space-y-10",
                            div { class: "terminal-block p-8 space-y-6",
                                h3 { class: "mono text-sm font-bold flex items-center gap-3",
                                    span { class: "text-[var(--accent)]", "#" }
                                    "ROOT_CA_SETTINGS"
                                }
                                p { class: "mono text-xs text-[var(--text-muted)] leading-relaxed",
                                    "Manage system-wide SSL trust for your local .local domains. Installing the CA requires administrative access via system security prompt."
                                }

                                div { class: "flex gap-4 pt-4",
                                    button {
                                        class: "btn-action px-8 py-3 mono text-xs font-bold",
                                        "data-tooltip": "Install and trust DevBind Root CA (elevated)",
                                        onclick: move |_| {
                                            let path = get_config_path();
                                            let mut dir = path.clone(); dir.pop();
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
                    }
                }
            }
        }
    }
}
