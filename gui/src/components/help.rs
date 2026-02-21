use dioxus::prelude::*;

#[component]
pub fn HelpTab() -> Element {
    rsx! {
        div { class: "max-w-4xl mx-auto pb-10",
            h1 { class: "text-2xl font-bold mb-8 text-white", "DevBind Help & Reference" }

            div { class: "space-y-10",

                // --- CLI Reference ---
                section {
                    h2 { class: "text-lg font-semibold mb-4 text-[var(--accent)] border-b border-[var(--border)] pb-2", "CLI Commands" }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { class: "terminal-block p-3",
                            code { class: "mono text-sm text-[var(--accent)] font-bold block mb-1", "devbind run <name> [cmd]" }
                            p { class: "text-xs text-[var(--text-muted)]", "Auto-allocates port, injects $PORT, proxies traffic, and runs dev server. Can auto-detect framework if [cmd] missing." }
                        }
                        div { class: "terminal-block p-3",
                            code { class: "mono text-sm text-[var(--accent)] font-bold block mb-1", "devbind add <name> <port>" }
                            p { class: "text-xs text-[var(--text-muted)]", "Manually maps <name>.test to a local <port>." }
                        }
                        div { class: "terminal-block p-3",
                            code { class: "mono text-sm text-[var(--accent)] font-bold block mb-1", "devbind start" }
                            p { class: "text-xs text-[var(--text-muted)]", "Starts the proxy server (HTTPS on 443, HTTP on 80)." }
                        }
                        div { class: "terminal-block p-3",
                            code { class: "mono text-sm text-[var(--accent)] font-bold block mb-1", "devbind gui" }
                            p { class: "text-xs text-[var(--text-muted)]", "Launches this visual interface." }
                        }
                        div { class: "terminal-block p-3",
                            code { class: "mono text-sm text-[var(--accent)] font-bold block mb-1", "devbind list" }
                            p { class: "text-xs text-[var(--text-muted)]", "Lists all active domain mappings." }
                        }
                        div { class: "terminal-block p-3",
                            code { class: "mono text-sm text-[var(--accent)] font-bold block mb-1", "devbind trust / untrust" }
                            p { class: "text-xs text-[var(--text-muted)]", "Installs/removes Root CA from system and browsers." }
                        }
                        div { class: "terminal-block p-3 md:col-span-2",
                            code { class: "mono text-sm text-[var(--accent)] font-bold block mb-1", "devbind install / uninstall" }
                            p { class: "text-xs text-[var(--text-muted)]", "Installs/removes systemd-resolved DNS integration for .test domains." }
                        }
                    }
                }

                // --- GUI Pages ---
                section {
                    h2 { class: "text-lg font-semibold mb-4 text-[var(--accent)] border-b border-[var(--border)] pb-2", "GUI Pages" }
                    div { class: "space-y-4",
                        div { class: "flex gap-4 p-3 bg-black/10 rounded border border-[var(--border)]",
                            div { class: "w-32 font-bold text-white mono whitespace-nowrap", "DASHBOARD" }
                            p { class: "text-sm text-[var(--text-muted)]", "View, add, and remove domain-to-port mappings in real-time. Useful for manual overrides." }
                        }
                        div { class: "flex gap-4 p-3 bg-black/10 rounded border border-[var(--border)]",
                            div { class: "w-32 font-bold text-white mono whitespace-nowrap", "DNS" }
                            p { class: "text-sm text-[var(--text-muted)]", "Check status and install the DevBind DNS resolver so *.test domains route instantly without editing /etc/hosts." }
                        }
                        div { class: "flex gap-4 p-3 bg-black/10 rounded border border-[var(--border)]",
                            div { class: "w-32 font-bold text-white mono whitespace-nowrap", "SSL TRUST" }
                            p { class: "text-sm text-[var(--text-muted)]", "Manage your local CA. Click here to banish 'Your connection is not private' browser warnings forever." }
                        }
                        div { class: "flex gap-4 p-3 bg-black/10 rounded border border-[var(--border)]",
                            div { class: "w-32 font-bold text-white mono whitespace-nowrap", "DAEMON" }
                            p { class: "text-sm text-[var(--text-muted)]", "Configure DevBind to run silently in the background via systemd so it starts automatically on login." }
                        }
                    }
                }

                // --- Troubleshooting ---
                section {
                    h2 { class: "text-lg font-semibold mb-4 text-[var(--accent)] border-b border-[var(--border)] pb-2", "Troubleshooting" }
                    ul { class: "list-disc list-inside space-y-3 text-sm text-[var(--text-muted)]",
                        li {
                            span { class: "font-semibold text-white", "Bad Gateway / Invalid Host: " },
                            "If you use Vite, React, or Vue, you must pass ",
                            span { class: "mono bg-black/30 px-1 rounded", "--host" },
                            " to the dev server to bind to IPv4 properly."
                        }
                        li {
                            span { class: "font-semibold text-white", "Connection Not Private: " },
                            "Go to the SSL TRUST tab and click Install Trust."
                        }
                        li {
                            span { class: "font-semibold text-white", "Site Cannot Be Reached: " },
                            "Go to the DNS tab and verify that the systemd-resolved integration is active and green."
                        }
                        li {
                            span { class: "font-semibold text-white", "Port 80/443 in Use: " },
                            "Ensure no other web servers (like Nginx, Apache, or another DevBind instance) are running. Stop them before starting DevBind."
                        }
                    }
                }
            }
        }
    }
}
