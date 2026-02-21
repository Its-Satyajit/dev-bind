use dioxus::prelude::*;

#[component]
pub fn DnsTab(
    mut dns_installed: Signal<bool>,
    mut error_msg: Signal<String>,
    mut success_msg: Signal<String>,
) -> Element {
    rsx! {
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
    }
}
