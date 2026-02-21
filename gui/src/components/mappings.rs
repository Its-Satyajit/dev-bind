use crate::utils::get_config_path;
use devbind_core::config::{DevBindConfig, RouteConfig};
use dioxus::prelude::*;

#[component]
pub fn MappingsTab(
    config: Signal<DevBindConfig>,
    mut new_domain: Signal<String>,
    mut new_port: Signal<String>,
    mut error_msg: Signal<String>,
    mut success_msg: Signal<String>,
) -> Element {
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

    rsx! {
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
                                else { cfg.routes.push(RouteConfig { domain: d, port: p }); }
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
    }
}
