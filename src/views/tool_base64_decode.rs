use base64::prelude::*;
use dioxus::prelude::*;
use strum::IntoEnumIterator;

use crate::clone;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum_macros::Display))]
enum Base64DecodeOutputKind {
    #[strum(to_string = "UTF-8")]
    Utf8,

    #[strum(to_string = "JSON")]
    Json,

    #[strum(to_string = "Hex")]
    SimpleHex,

    #[strum(to_string = "Pretty Hex")]
    PrettyHex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolBase64DecodeComponentData {
    input: String,
    output_kind: Base64DecodeOutputKind,
}

impl Default for ToolBase64DecodeComponentData {
    fn default() -> Self {
        Self {
            input: Default::default(),
            output_kind: Base64DecodeOutputKind::Utf8,
        }
    }
}

#[component]
pub fn ToolBase64DecodeComponent(
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
                    .map(|v: serde_json::Value| {
                        serde_json::to_string_pretty(&v).unwrap_or(s.clone())
                    })
                    .unwrap_or(s)
            }
            Base64DecodeOutputKind::SimpleHex => hex::encode(&v),
            Base64DecodeOutputKind::PrettyHex => pretty_hex::pretty_hex(&v),
        })
        .map_or((String::new(), false), |v| (v, true));

    let output_kinds = Base64DecodeOutputKind::iter().map(|v| {
        let id = format!(
            "ToolBase64DecodeComponent-btn-radio-output-{}",
            Base64DecodeOutputKindDiscriminants::from(v)
        );
        clone!(data);
        rsx! {
            input {
                checked: data.output_kind == v,
                class: "btn-check",
                id: id.clone(),
                r#type: "radio",
                onchange: move |_| {
                    clone!(data);
                    onupdate.call(ToolBase64DecodeComponentData{
                        output_kind: v,
                        ..data
                    });
                }
            }
            label { class: "btn btn-outline-primary", r#for: id, "{v}" }
        }
    });

    rsx! {
        div { class: "d-flex flex-column m-1",
            div { class: "d-flex mb-1",
                h5 { "Input" }
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
                    { output_kinds }
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
