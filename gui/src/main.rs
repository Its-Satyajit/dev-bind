use devbind_core::config::{DevBindConfig, RouteConfig};
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

    // Define normal function to avoid FnMut closure cloning issues
    let update_config = move |cfg: DevBindConfig,
                              mut config_sig: Signal<DevBindConfig>,
                              mut err_sig: Signal<String>| {
        let path = get_config_path();
        if let Err(e) = cfg.save(&path) {
            err_sig.set(format!("Failed to save config: {}", e));
            return;
        }

        // Try to update hosts
        let hosts_path = PathBuf::from("/etc/hosts");
        let manager = HostsManager::new(&hosts_path);
        let domains: Vec<String> = cfg.routes.iter().map(|r| r.domain.clone()).collect();

        if let Err(e) = manager.update_routes(&domains) {
            err_sig.set(format!(
                "Warning: Failed to edit /etc/hosts (try running GUI with sudo): {}",
                e
            ));
        } else {
            err_sig.set(String::new());
        }
        config_sig.set(cfg);
    };

    rsx! {
        link { rel: "stylesheet", href: "https://unpkg.com/tailwindcss@2.2.19/dist/tailwind.min.css" }
        div { class: "min-h-screen bg-gray-100 p-8",
            div { class: "max-w-4xl mx-auto",
                div { class: "bg-white rounded-lg shadow-lg p-6",
                    div { class: "flex justify-between items-center mb-6",
                        h1 { class: "text-2xl font-bold text-gray-800", "DevBind Dashboard" }
                        button {
                            class: "bg-green-600 text-white px-4 py-2 rounded shadow hover:bg-green-700 flex items-center gap-2",
                            onclick: move |_| {
                                let path = get_config_path();
                                let mut dir = path.clone();
                                dir.pop();

                                match devbind_core::trust::install_root_ca(&dir) {
                                    Ok(_) => {
                                        success_msg.set("Root CA Trusted Successfully!".to_string());
                                        error_msg.set(String::new());
                                    },
                                    Err(e) => {
                                        error_msg.set(format!("Trust failed: {}", e));
                                        success_msg.set(String::new());
                                    }
                                }
                            },
                            "🔒 Trust Root CA"
                        }
                    }

                    if !error_msg().is_empty() {
                        div { class: "mb-4 p-4 text-sm text-red-700 bg-red-100 rounded-lg", "{error_msg()}" }
                    }
                    if !success_msg().is_empty() {
                        div { class: "mb-4 p-4 text-sm text-green-700 bg-green-100 rounded-lg", "{success_msg()}" }
                    }

                    div { class: "grid gap-6",
                        // Add New Mapping Form
                        div { class: "bg-gray-50 p-4 rounded border flex flex-col md:flex-row gap-4 items-end",
                            div { class: "flex-1",
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "Domain" }
                                input {
                                    class: "w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 p-2 border",
                                    placeholder: "my-app.dev.local",
                                    value: "{new_domain()}",
                                    oninput: move |evt| new_domain.set(evt.value().clone())
                                }
                            }
                            div { class: "w-32",
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "Port" }
                                input {
                                    class: "w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 p-2 border",
                                    placeholder: "3000",
                                    value: "{new_port()}",
                                    oninput: move |evt| new_port.set(evt.value().clone())
                                }
                            }
                            button {
                                class: "bg-indigo-600 text-white px-4 py-2 rounded shadow hover:bg-indigo-700",
                                onclick: move |_| {
                                    let mut cfg = config();
                                    if let Ok(port) = new_port().parse::<u16>() {
                                        let domain = new_domain();
                                        if !domain.is_empty() {
                                            if let Some(route) = cfg.routes.iter_mut().find(|r| r.domain == domain) {
                                                route.port = port;
                                            } else {
                                                cfg.routes.push(RouteConfig { domain, port });
                                            }
                                            update_config(cfg, config, error_msg);
                                            new_domain.set(String::new());
                                            new_port.set(String::new());
                                        }
                                    } else {
                                        error_msg.set("Invalid port number".to_string());
                                    }
                                },
                                "Add Mapping"
                            }
                        }

                        // Mappings Section
                        div { class: "bg-gray-50 p-4 rounded border",
                            h2 { class: "text-lg font-semibold mb-4 text-gray-700", "Local Domains" }
                            if config().routes.is_empty() {
                                p { class: "text-gray-500 italic", "No domains configured yet." }
                            } else {
                                table { class: "min-w-full divide-y divide-gray-200",
                                    thead {
                                        tr {
                                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Domain" }
                                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Port" }
                                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Status" }
                                            th { class: "px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider", "Actions" }
                                        }
                                    }
                                    tbody { class: "bg-white divide-y divide-gray-200",
                                        for route in config().routes {
                                            tr {
                                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900", "{route.domain}" }
                                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{route.port}" }
                                                td { class: "px-6 py-4 whitespace-nowrap",
                                                    span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800",
                                                        "Active"
                                                    }
                                                }
                                                td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                                                    button {
                                                        class: "text-red-600 hover:text-red-900",
                                                        onclick: move |_| {
                                                            let mut cfg = config();
                                                            let d = route.domain.clone();
                                                            cfg.routes.retain(|r| r.domain != d);
                                                            update_config(cfg, config, error_msg);
                                                        },
                                                        "Delete"
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
    }
}
