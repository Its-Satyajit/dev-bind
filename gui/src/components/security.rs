use crate::utils::get_config_dir;
use dioxus::prelude::*;

#[component]
pub fn SecurityTab(mut error_msg: Signal<String>, mut success_msg: Signal<String>) -> Element {
    rsx! {
        div { class: "max-w-2xl mx-auto space-y-10",
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
    }
}
