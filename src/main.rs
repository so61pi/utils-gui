use std::{cell::Cell, rc::Rc};

use base64::engine::general_purpose::URL_SAFE;
use base64::prelude::*;
use dioxus::{logger::tracing, prelude::*};
use strum::IntoEnumIterator;
mod utils;

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
    JWT,
    Certificate,
    DateTime,
    IP,
}

#[component]
pub fn Home() -> Element {
    let mut selected_tool = use_signal(|| Tools::Base64Encode);

    let mut data_tool_base64_encode = use_signal(|| ToolBase64EncodeComponentData {
        input: String::new(),
        input_kind: Base64EncodeInputKind::Utf8,
        output_kind: Base64EncodeOutputKind::Standard,
    });
    let mut data_tool_base64_decode = use_signal(|| ToolBase64DecodeComponentData {
        input: String::new(),
        output_kind: Base64DecodeOutputKind::Utf8,
    });

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

                    style: "flex: 3;",

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
    let mut theme_light_mode = use_signal(|| true);
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum Base64EncodeInputKind {
    Utf8,
    Hex,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Base64EncodeOutputKind {
    Standard,
    UrlSafe,
}

#[derive(Debug, Clone, PartialEq)]
struct ToolBase64EncodeComponentData {
    input: String,
    input_kind: Base64EncodeInputKind,
    output_kind: Base64EncodeOutputKind,
}

#[component]
fn ToolBase64EncodeComponent(
    data: ToolBase64EncodeComponentData,
    onupdate: EventHandler<ToolBase64EncodeComponentData>,
) -> Element {
    let input = match data.input_kind {
        Base64EncodeInputKind::Utf8 => Ok(data.input.as_bytes().to_vec()),
        Base64EncodeInputKind::Hex => {
            let input = data.input.replace([' ', '\t'], "");
            hex::decode(&input)
        }
    };
    let (output, input_valid) = input
        .map(|v| match data.output_kind {
            Base64EncodeOutputKind::Standard => BASE64_STANDARD.encode(v),
            Base64EncodeOutputKind::UrlSafe => URL_SAFE.encode(v),
        })
        .map_or((String::new(), false), |v| (v, true));

    rsx! {
        div { class: "d-flex flex-column m-1",
            div { class: "d-flex mb-1",
                h5 { "Input" }
                div {
                    class: "btn-group ms-auto",
                    role: "group",
                    input {
                        checked: data.input_kind == Base64EncodeInputKind::Utf8,
                        class: "btn-check",
                        id: "ToolBase64EncodeComponent-btn-radio-input-utf8",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64EncodeComponentData{
                                    input_kind: Base64EncodeInputKind::Utf8,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64EncodeComponent-btn-radio-input-utf8", "UTF-8" }

                    input {
                        checked: data.input_kind == Base64EncodeInputKind::Hex,
                        class: "btn-check",
                        id: "ToolBase64EncodeComponent-btn-radio-input-simplehex",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64EncodeComponentData{
                                    input_kind: Base64EncodeInputKind::Hex,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64EncodeComponent-btn-radio-input-simplehex", "Hex" }
                }
            }
            textarea {
                "autocorrect": "off",
                "autocapitalize": "none",
                class: format!("font-monospace form-control {}", if !input_valid { "border-danger" } else { "" }),
                rows: "3",
                oninput: {
                    clone!(data);
                    move |v: Event<FormData>| {
                        clone!(data);
                        onupdate.call(ToolBase64EncodeComponentData{
                            input: v.value(),
                            ..data
                        });
                    }
                },
                { data.input.clone() }
            }

            hr {}

            div { class: "d-flex mb-1",
                h5 { "Output" }
                div {
                    class: "btn-group ms-auto",
                    role: "group",
                    input {
                        checked: data.output_kind == Base64EncodeOutputKind::Standard,
                        class: "btn-check",
                        id: "ToolBase64EncodeComponent-btn-radio-output-standard",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64EncodeComponentData{
                                    output_kind: Base64EncodeOutputKind::Standard,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64EncodeComponent-btn-radio-output-standard", "Standard" }

                    input {
                        checked: data.output_kind == Base64EncodeOutputKind::UrlSafe,
                        class: "btn-check",
                        id: "ToolBase64EncodeComponent-btn-radio-output-urlsafe",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64EncodeComponentData{
                                    output_kind: Base64EncodeOutputKind::UrlSafe,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64EncodeComponent-btn-radio-output-urlsafe", "URL" }
                }
            }
            textarea {
                readonly: true,
                class: "font-monospace form-control",
                rows: "3",
                { output }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Base64DecodeOutputKind {
    Utf8,
    Json,
    SimpleHex,
    PrettyHex,
}

#[derive(Debug, Clone, PartialEq)]
struct ToolBase64DecodeComponentData {
    input: String,
    output_kind: Base64DecodeOutputKind,
}

#[component]
fn ToolBase64DecodeComponent(
    data: ToolBase64DecodeComponentData,
    onupdate: EventHandler<ToolBase64DecodeComponentData>,
) -> Element {
    let decoded = BASE64_STANDARD.decode(&data.input);
    let (output, input_valid) = decoded
        .map(|v| match data.output_kind {
            Base64DecodeOutputKind::Utf8 => String::from_utf8_lossy(&v).to_string(),
            Base64DecodeOutputKind::Json => {
                let s = String::from_utf8_lossy(&v).to_string();
                serde_json::from_str(&s)
                    .map(|v: serde_json::Value| serde_json::to_string_pretty(&v))
                    .unwrap_or_else(|_| Ok(s.clone()))
                    .unwrap_or_else(|_| s)
            },
            Base64DecodeOutputKind::SimpleHex => {
                let mut cfg = pretty_hex::HexConfig::simple();
                cfg.group = 0;
                pretty_hex::config_hex(&v, cfg)
            }
            Base64DecodeOutputKind::PrettyHex => pretty_hex::pretty_hex(&v),
        })
        .map_or((String::new(), false), |v| (v, true));

    rsx! {
        div { class: "d-flex flex-column m-1",
            h5 { "Input" }
            textarea {
                "autocorrect": "off",
                "autocapitalize": "none",
                class: format!("font-monospace form-control {}", if !input_valid { "border-danger" } else { "" }),
                rows: "3",
                oninput: {
                    clone!(data);
                    move |v: Event<FormData>| {
                        clone!(data);
                        onupdate.call(ToolBase64DecodeComponentData{
                            input: v.value(),
                            ..data
                        });
                    }
                },
                { data.input.clone() }
            }

            hr {}

            div { class: "d-flex mb-1",
                h5 { "Output" }
                div {
                    class: "btn-group ms-auto",
                    role: "group",
                    input {
                        checked: data.output_kind == Base64DecodeOutputKind::Utf8,
                        class: "btn-check",
                        id: "ToolBase64DecodeComponent-btn-radio-utf8",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64DecodeComponentData{
                                    output_kind:Base64DecodeOutputKind::Utf8,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64DecodeComponent-btn-radio-utf8", "UTF-8" }

                    input {
                        checked: data.output_kind == Base64DecodeOutputKind::Json,
                        class: "btn-check",
                        id: "ToolBase64DecodeComponent-btn-radio-json",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64DecodeComponentData{
                                    output_kind:Base64DecodeOutputKind::Json,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64DecodeComponent-btn-radio-json", "JSON" }

                    input {
                        checked: data.output_kind == Base64DecodeOutputKind::SimpleHex,
                        class: "btn-check",
                        id: "ToolBase64DecodeComponent-btn-radio-simplehex",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64DecodeComponentData{
                                    output_kind:Base64DecodeOutputKind::SimpleHex,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64DecodeComponent-btn-radio-simplehex", "Hex" }

                    input {
                        checked: data.output_kind == Base64DecodeOutputKind::PrettyHex,
                        class: "btn-check",
                        id: "ToolBase64DecodeComponent-btn-radio-prettyhex",
                        r#type: "radio",
                        onchange: {
                            clone!(data);
                            move |_| {
                                clone!(data);
                                onupdate.call(ToolBase64DecodeComponentData{
                                    output_kind:Base64DecodeOutputKind::PrettyHex,
                                    ..data
                                });
                            }
                        }
                    }
                    label { class: "btn btn-outline-primary", r#for: "ToolBase64DecodeComponent-btn-radio-prettyhex", "Pretty Hex" }
                }
            }
            textarea {
                readonly: true,
                class: "font-monospace form-control",
                rows: "3",
                { output }
            }
        }
    }
}
