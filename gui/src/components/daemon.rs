use crate::service::{install_service, is_service_active, is_service_installed, uninstall_service};
use crate::utils::{is_proxy_running, which_devbind};
use dioxus::prelude::*;

#[component]
pub fn DaemonTab(
    mut proxy_online: Signal<bool>,
    mut service_installed: Signal<bool>,
    mut service_active: Signal<bool>,
    mut error_msg: Signal<String>,
    mut success_msg: Signal<String>,
) -> Element {
    rsx! {
        div { class: "max-w-2xl mx-auto space-y-8",

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
                            if service_active() { "SERVICE_ACTIVE" } else if proxy_online() { "SERVICE_INACTIVE (Proxy running manually)" } else { "SERVICE_INACTIVE" }
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
