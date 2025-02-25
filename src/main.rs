use std::{cell::Cell, rc::Rc};

use dioxus::{logger::tracing, prelude::*};
use strum::IntoEnumIterator;
use views::{
    tool_base64_decode::{ToolBase64DecodeComponent, ToolBase64DecodeComponentData},
    tool_base64_encode::{ToolBase64EncodeComponent, ToolBase64EncodeComponentData},
    tool_base64_hash::{ToolHashComponent, ToolHashComponentData},
};

mod utils;
mod views;

fn main() -> anyhow::Result<()> {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(make_config())
        .launch(app);

    Ok(())
}

fn app() -> Element {
    let zoom_level = use_hook(|| Rc::new(Cell::new(1.0)));

    use_future(move || {
        clone!(zoom_level);
        async move {
            // From: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent
            #[allow(non_snake_case)]
            #[allow(dead_code)]
            #[derive(Debug, Clone, Default, serde::Deserialize)]
            struct KeyboardEvent {
                altKey: bool,
                /// Returns a string with the code value of the physical key represented by the event.
                code: String,
                /// Returns a boolean value that is true if the Ctrl key was active when the key event was generated.
                ctrlKey: bool,
                isComposing: bool,
                /// Returns a string representing the key value of the key represented by the event.
                key: String,
                location: u32,
                /// Returns a boolean value that is true if the Meta key (on Mac keyboards, the ⌘ Command key;
                /// on Windows keyboards, the Windows key (⊞)) was active when the key event was generated.
                metaKey: bool,
                repeat: bool,
                shiftKey: bool,
            }
            impl KeyboardEvent {
                fn ctrl_or_command(&self) -> bool {
                    #[cfg(not(target_os = "macos"))]
                    {
                        self.ctrlKey
                    }

                    #[cfg(target_os = "macos")]
                    {
                        self.metaKey
                    }
                }
            }

            let mut eval =
                document::eval("$(document).ready(function() { setupKeydownEvent(dioxus); });");
            loop {
                match eval.recv::<KeyboardEvent>().await {
                    Ok(event) => {
                        tracing::debug!("keypress {event:?}");
                        if event.ctrl_or_command() && event.key == "=" {
                            tracing::debug!("zoom in");
                            zoom_level.replace(zoom_level.get() + 0.1);
                            _ = dioxus::desktop::window().webview.zoom(zoom_level.get());
                        } else if event.ctrl_or_command() && event.key == "-" {
                            tracing::debug!("zoom out");
                            zoom_level.replace(zoom_level.get() - 0.1);
                            _ = dioxus::desktop::window().webview.zoom(zoom_level.get());
                        } else if event.ctrl_or_command() && event.key == "0" {
                            tracing::debug!("reset zoom");
                            zoom_level.replace(1.0);
                            _ = dioxus::desktop::window().webview.zoom(zoom_level.get());
                        }
                    }
                    Err(error) => {
                        tracing::error!("watching keydown event failed with {error}");
                        break;
                    }
                }
            }
        }
    });

    rsx! {
        Home {}
    }
}

fn make_config() -> dioxus::desktop::Config {
    dioxus::desktop::Config::default()
        .with_window(make_window())
        // .with_close_behaviour(dioxus::desktop::WindowCloseBehaviour::LastWindowExitsApp)
        .with_custom_index(
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/index.html")).into(),
        )
}

fn make_window() -> dioxus::desktop::WindowBuilder {
    dioxus::desktop::WindowBuilder::new()
        .with_always_on_top(false)
        .with_title("Utils")
}

#[derive(Debug, Clone, PartialEq, strum_macros::Display, strum_macros::EnumIter)]
enum Tools {
    #[strum(to_string = "Base64 Encode")]
    Base64Encode,

    #[strum(to_string = "Base64 Decode")]
    Base64Decode,

    Hash,
    Jwt,
    Certificate,
    DateTime,
    IP,
}

#[component]
pub fn Home() -> Element {
    let mut selected_tool = use_signal(|| Tools::Base64Encode);

    let mut data_tool_base64_encode = use_signal(ToolBase64EncodeComponentData::default);
    let mut data_tool_base64_decode = use_signal(ToolBase64DecodeComponentData::default);
    let mut data_tool_hash = use_signal(ToolHashComponentData::default);

    let selected_tool_ui = match selected_tool() {
        Tools::Base64Encode => rsx! {
            ToolBase64EncodeComponent {
                data: data_tool_base64_encode(),
                onupdate: move |v| data_tool_base64_encode.set(v),
            }
        },
        Tools::Base64Decode => rsx! {
            ToolBase64DecodeComponent {
                data: data_tool_base64_decode(),
                onupdate: move |v| data_tool_base64_decode.set(v),
            }
        },
        Tools::Hash => rsx! {
            ToolHashComponent {
                data: data_tool_hash(),
                onupdate: move |v| data_tool_hash.set(v),
            }
        },
        _ => rsx! {
            "TODO"
        },
    };

    rsx! {
        div {
            class: "d-flex flex-column",
            style: "width: 100vw; height: 100vh;",

            div { class: "bg-primary-subtle",
                style: "flex: none;",
                "🛠️ Utils"
            }

            div { class: "flex-grow-1 d-flex overflow-auto h-100",

                div {
                    class: "d-flex flex-column overflow-auto m-1",
                    // Need `overflow: auto;` with the same reason as above.
                    // See https://stackoverflow.com/questions/36247140/why-dont-flex-items-shrink-past-content-size

                    style: "flex: 3; max-width: 250px;",

                    ToolSelectorComponent {
                        selected: selected_tool(),
                        onchange: move |selected| selected_tool.set(selected),
                    }
                }

                div { class: "lv-resizer lv-resizer-h" }
                div { class: "d-flex flex-column overflow-auto",
                    style: "flex: 7;",
                    { selected_tool_ui }
                }
            }

            div { class: "bg-body-secondary d-flex",
                style: "flex: none;",
                StatusComponent {}
            }
        }
    }
}

#[component]
fn StatusComponent() -> Element {
    let mut theme_light_mode = use_signal(|| false);
    use_effect(move || {
        let mode = theme_light_mode();
        spawn(async move {
            let js = format!(r#"setTheme({})"#, mode);
            if let Err(e) = document::eval(&js).await {
                tracing::error!("running javascript failed: {e}");
            }
        });
    });

    rsx! {
        div { class: "form-check form-switch ms-auto",
            input {
                class: "form-check-input",
                id: "ThemeSelector",
                r#type: "checkbox",
                role: "switch",
                checked: !theme_light_mode(),
                onclick: move |_| theme_light_mode.toggle(),
            }
            label { class: "form-check-label", r#for: "ThemeSelector",
                "🌒"
            }
        }
    }
}

#[component]
fn ToolSelectorComponent(selected: Tools, onchange: EventHandler<Tools>) -> Element {
    let tools = Tools::iter().map(|tool| rsx! {
        button { class: format!("btn btn-block {}", if selected == tool { "btn-primary" } else { "" }),
            onclick: move |_| {
                clone!(tool);
                onchange.call(tool)
            },
            { tool.to_string() }
        }
    });

    rsx! {
        div {
            class: "d-flex flex-column",
            { tools }
        }
    }
}
