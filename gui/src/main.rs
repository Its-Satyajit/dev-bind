use devbind_core::config::{AppTheme, DevBindConfig, RouteConfig};
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
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let config = use_signal(|| DevBindConfig::load(&get_config_path()).unwrap_or_default());
    let mut new_domain = use_signal(|| String::new());
    let mut new_port = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());
    let mut success_msg = use_signal(|| String::new());
    let mut active_tab = use_signal(|| "dashboard");

    let current_theme = config().ui.theme;

    let update_config = move |mut cfg: DevBindConfig,
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

    let theme_vars = match current_theme {
        AppTheme::BreezeDark => (
            "#232629", "#31363b", "#2a2e32", "#eff0f1", "#7f8c8d", "#3daee9", "#1d99f3", "#4d4d4d",
            "2px",
        ),
        AppTheme::BreezeLight => (
            "#eff0f1", "#fcfcfc", "#ffffff", "#232629", "#7f8c8d", "#3daee9", "#1d99f3", "#cdd3da",
            "2px",
        ),
        AppTheme::AdwaitaDark => (
            "#1e1e1e", "#2d2d2d", "#242424", "#ffffff", "#9a9996", "#3584e4", "#1c71d8", "#303030",
            "8px",
        ),
        AppTheme::AdwaitaLight => (
            "#f6f5f4", "#ebebeb", "#ffffff", "#2e3436", "#5e5c64", "#3584e4", "#1c71d8", "#d6d6d6",
            "8px",
        ),
    };

    let style_content = format!(
        r#"
        @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
        @import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap');
        :root {{
            --bg-main: {0}; --bg-sidebar: {1}; --bg-card: {2}; --text-main: {3}; --text-muted: {4};
            --accent: {5}; --accent-hover: {6}; --border: {7}; --radius: {8};
        }}
        body {{
            font-family: 'Inter', sans-serif; background-color: var(--bg-main); color: var(--text-main);
            transition: background-color 0.2s ease; margin: 0;
        }}
        .mono {{ font-family: 'JetBrains Mono', monospace; }}
        .sidebar {{ background-color: var(--bg-sidebar); border-right: 1px solid var(--border); }}
        .terminal-block {{
            background-color: rgba(0, 0, 0, 0.05); border: 1px solid var(--border); border-radius: var(--radius);
        }}
        .btn-action {{
            background-color: var(--accent); color: white; border-radius: var(--radius); border: none; cursor: pointer;
        }}
        .btn-action:hover {{ background-color: var(--accent-hover); }}
        "#,
        theme_vars.0,
        theme_vars.1,
        theme_vars.2,
        theme_vars.3,
        theme_vars.4,
        theme_vars.5,
        theme_vars.6,
        theme_vars.7,
        theme_vars.8
    );

    let dashboard_active = active_tab() == "dashboard";
    let security_active = active_tab() == "security";

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
                        class: if dashboard_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:bg-white/5 transition-all" },
                        onclick: move |_| active_tab.set("dashboard"),
                        "DASHBOARD"
                    }
                    button {
                        class: if security_active { "w-full text-left px-8 py-3 bg-[var(--accent)] text-white font-medium" } else { "w-full text-left px-8 py-3 text-[var(--text-muted)] hover:bg-white/5 transition-all" },
                        onclick: move |_| active_tab.set("security"),
                        "SECURITY"
                    }
                }

                div { class: "p-8 space-y-4 border-t border-[var(--border)]",
                    p { class: "text-[9px] font-bold text-[var(--text-muted)] uppercase tracking-widest", "THEME" }
                    div { class: "space-y-1",
                        button {
                            class: if current_theme == AppTheme::BreezeDark { "w-full text-left text-[10px] py-1 px-2 rounded bg-white/10" } else { "w-full text-left text-[10px] py-1 px-2 rounded text-[var(--text-muted)]" },
                            onclick: move |_| { let mut cfg = config(); cfg.ui.theme = AppTheme::BreezeDark; update_config(cfg, config, error_msg); },
                            "[ KONSOLE DARK ]"
                        }
                        button {
                            class: if current_theme == AppTheme::BreezeLight { "w-full text-left text-[10px] py-1 px-2 rounded bg-white/10" } else { "w-full text-left text-[10px] py-1 px-2 rounded text-[var(--text-muted)]" },
                            onclick: move |_| { let mut cfg = config(); cfg.ui.theme = AppTheme::BreezeLight; update_config(cfg, config, error_msg); },
                            "[ KONSOLE LIGHT ]"
                        }
                        button {
                            class: if current_theme == AppTheme::AdwaitaDark { "w-full text-left text-[10px] py-1 px-2 rounded bg-white/10" } else { "w-full text-left text-[10px] py-1 px-2 rounded text-[var(--text-muted)]" },
                            onclick: move |_| { let mut cfg = config(); cfg.ui.theme = AppTheme::AdwaitaDark; update_config(cfg, config, error_msg); },
                            "[ GNOME DARK ]"
                        }
                        button {
                            class: if current_theme == AppTheme::AdwaitaLight { "w-full text-left text-[10px] py-1 px-2 rounded bg-white/10" } else { "w-full text-left text-[10px] py-1 px-2 rounded text-[var(--text-muted)]" },
                            onclick: move |_| { let mut cfg = config(); cfg.ui.theme = AppTheme::AdwaitaLight; update_config(cfg, config, error_msg); },
                            "[ GNOME LIGHT ]"
                        }
                    }
                }
            }

            main { class: "flex-1 flex flex-col bg-[var(--bg-main)]",
                header { class: "px-10 py-6 border-b border-[var(--border)] flex justify-between items-center",
                    h2 { class: "text-xs font-bold mono text-[var(--text-muted)]", "{active_tab().to_uppercase()}" }
                    div { class: "mono text-[10px]",
                        if !success_msg().is_empty() { span { class: "text-green-500", "SUCCESS: {success_msg()}" } }
                        if !error_msg().is_empty() { span { class: "text-red-500", "ERROR: {error_msg()}" } }
                    }
                }

                div { class: "p-10 flex-1 overflow-y-auto",
                    if dashboard_active {
                        div { class: "max-w-4xl space-y-10",
                            div { class: "flex items-center gap-4 bg-[var(--bg-sidebar)] p-2 rounded border border-[var(--border)]",
                                span { class: "mono text-[var(--accent)] ml-4", "ADD>" }
                                input {
                                    class: "flex-1 bg-transparent border-none text-sm px-4 py-2 mono text-[var(--text-main)]",
                                    placeholder: "host.local",
                                    value: "{new_domain()}",
                                    oninput: move |e| new_domain.set(e.value().clone())
                                }
                                input {
                                    class: "w-24 bg-transparent border-none text-sm px-4 py-2 mono text-[var(--text-main)]",
                                    placeholder: "3000",
                                    value: "{new_port()}",
                                    oninput: move |e| new_port.set(e.value().clone())
                                }
                                button {
                                    class: "btn-action px-6 py-2 text-xs font-bold mono",
                                    onclick: move |_| {
                                        let mut cfg = config();
                                        if let Ok(p) = new_port().parse::<u16>() {
                                            let mut d = new_domain();
                                            if !d.is_empty() {
                                                if !d.ends_with(".local") { d.push_str(".local"); }
                                                if let Some(r) = cfg.routes.iter_mut().find(|r| r.domain == d) { r.port = p; }
                                                else { cfg.routes.push(RouteConfig { domain: d, port: p }); }
                                                update_config(cfg, config, error_msg);
                                                new_domain.set(String::new());
                                                new_port.set(String::new());
                                                success_msg.set("SYNCED".to_string());
                                            }
                                        } else { error_msg.set("INVALID_PORT".to_string()); }
                                    },
                                    "DEPLOY"
                                }
                            }

                            div { class: "terminal-block overflow-hidden",
                                div { class: "bg-white/5 px-8 py-3 border-b border-[var(--border)] flex justify-between",
                                    span { class: "mono text-[9px] font-bold text-[var(--text-muted)]", "INFRASTRUCTURE_STATE" }
                                    span { class: "mono text-[9px] text-[var(--text-muted)]", "ACTIVE: {config().routes.len()}" }
                                }
                                div { class: "p-4",
                                    if config().routes.is_empty() { p { class: "mono text-xs text-[var(--text-muted)] p-4", "# No routes configured." } }
                                    else {
                                        table { class: "w-full mono text-xs",
                                            tbody {
                                                for r in config().routes {
                                                    tr { class: "hover:bg-white/5 group",
                                                        td { class: "px-4 py-3", span { class: "text-[var(--accent)] mr-2", ">" } "{r.domain}" }
                                                        td { class: "px-4 py-3 text-[var(--text-muted)]", "127.0.0.1:{r.port}" }
                                                        td { class: "px-4 py-3 text-right",
                                                            button {
                                                                class: "text-red-500 opacity-0 group-hover:opacity-100 transition-all px-2",
                                                                onclick: move |_| {
                                                                    let mut cfg = config();
                                                                    let d = r.domain.clone();
                                                                    cfg.routes.retain(|x| x.domain != d);
                                                                    update_config(cfg, config, error_msg);
                                                                    success_msg.set("REMOVED".to_string());
                                                                },
                                                                "[X]"
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
                    } else if security_active {
                        div { class: "max-w-2xl space-y-10",
                            div { class: "terminal-block p-8 space-y-6",
                                h3 { class: "mono text-sm font-bold", "ROOT_CA_ORCHESTRATION" }
                                p { class: "mono text-xs text-[var(--text-muted)] leading-relaxed",
                                    "Establishing system-wide trust for local development certificates. This procedure requires escalated privileges."
                                }
                                div { class: "flex gap-4 pt-4",
                                    button {
                                        class: "btn-action px-8 py-3 mono text-xs font-bold",
                                        onclick: move |_| {
                                            let path = get_config_path();
                                            let mut dir = path.clone(); dir.pop();
                                            match devbind_core::trust::install_root_ca(&dir) {
                                                Ok(_) => { success_msg.set("TRUST_OK".to_string()); error_msg.set(String::new()); },
                                                Err(e) => { error_msg.set(format!("FAIL: {}", e)); success_msg.set(String::new()); }
                                            }
                                        },
                                        "EXEC_INSTALL"
                                    }
                                    button {
                                        class: "border border-red-500/20 text-red-500/60 px-8 py-3 mono text-xs font-bold rounded",
                                        onclick: move |_| {
                                            match devbind_core::trust::uninstall_root_ca() {
                                                Ok(_) => { success_msg.set("TRUST_REVOKED".to_string()); error_msg.set(String::new()); },
                                                Err(e) => { error_msg.set(format!("FAIL: {}", e)); success_msg.set(String::new()); }
                                            }
                                        },
                                        "EXEC_UNINSTALL"
                                    }
                                }
                            }
                            p { class: "mono text-[10px] text-amber-500/40 px-4", "# NOTE: Administrative authentication will be required." }
                        }
                    }
                }
            }
        }
    }
}
