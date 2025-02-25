use base64::engine::general_purpose::URL_SAFE;
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
enum Base64EncodeInputKind {
    #[strum(to_string = "UTF-8")]
    Utf8,
    Hex,
}

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
enum Base64EncodeOutputKind {
    Standard,

    #[strum(to_string = "URL")]
    UrlSafe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolBase64EncodeComponentData {
    input: String,
    input_kind: Base64EncodeInputKind,
    output_kind: Base64EncodeOutputKind,
}

impl Default for ToolBase64EncodeComponentData {
    fn default() -> Self {
        Self {
            input: Default::default(),
            input_kind: Base64EncodeInputKind::Utf8,
            output_kind: Base64EncodeOutputKind::Standard,
        }
    }
}

#[component]
pub fn ToolBase64EncodeComponent(
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

    let input_kinds = Base64EncodeInputKind::iter().map(|v| {
        let id = format!(
            "ToolBase64EncodeComponent-btn-radio-input-{}",
            Base64EncodeInputKindDiscriminants::from(v)
        );
        clone!(data);
        rsx! {
            input {
                checked: data.input_kind == v,
                class: "btn-check",
                id: id.clone(),
                r#type: "radio",
                onchange: move |_| {
                    clone!(data);
                    onupdate.call(ToolBase64EncodeComponentData{
                        input_kind: v,
                        ..data
                    });
                }
            }
            label { class: "btn btn-outline-primary", r#for: id, "{v}" }
        }
    });
    let output_kinds = Base64EncodeOutputKind::iter().map(|v| {
        let id = format!(
            "ToolBase64EncodeComponent-btn-radio-output-{}",
            Base64EncodeOutputKindDiscriminants::from(v)
        );
        clone!(data);
        rsx! {
            input {
                checked: data.output_kind == v,
                class: "btn-check",
                id: id.clone(),
                r#type: "radio",
                onchange: {
                    clone!(data);
                    move |_| {
                        clone!(data);
                        onupdate.call(ToolBase64EncodeComponentData{
                            output_kind: v,
                            ..data
                        });
                    }
                }
            }
            label { class: "btn btn-outline-primary", r#for: id, "{v}" }
        }
    });

    rsx! {
        div { class: "d-flex flex-column m-1",
            div { class: "d-flex mb-1",
                h5 { "Input" }
                div {
                    class: "btn-group ms-auto",
                    role: "group",
                    { input_kinds }
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
