use dioxus::prelude::*;
use devbind_core::config::DevBindConfig;
use std::path::PathBuf;

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    path.push("devbind");
    path.push("config.toml");
    path
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let config = use_signal(|| DevBindConfig::load(&get_config_path()).unwrap_or_default());

    rsx! {
        link { rel: "stylesheet", href: "https://unpkg.com/tailwindcss@2.2.19/dist/tailwind.min.css" }
        div { class: "min-h-screen bg-gray-100 p-8",
            div { class: "max-w-4xl mx-auto",
                div { class: "bg-white rounded-lg shadow-lg p-6",
                    h1 { class: "text-2xl font-bold mb-6 text-gray-800", "DevBind Dashboard" }

                    div { class: "grid gap-6",
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
                                        }
                                    }
                                    tbody { class: "bg-white divide-y divide-gray-200",
                                        for route in &config().routes {
                                            tr {
                                                td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900", "{route.domain}" }
                                                td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{route.port}" }
                                                td { class: "px-6 py-4 whitespace-nowrap",
                                                    span { class: "px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800",
                                                        "Active"
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
