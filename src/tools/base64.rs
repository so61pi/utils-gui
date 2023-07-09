use base64::Engine;
use dioxus::prelude::*;

const TOOLID: &str = "tool-base64";

#[derive(Default)]
pub struct ToolData {
    input: String,
}

pub fn ToolBase64(cx: Scope) -> Element {
    let data = use_shared_state::<ToolData>(cx).unwrap();
    let dr = data.read();

    // Encode input data
    let encoded = base64::engine::general_purpose::STANDARD_NO_PAD.encode(dr.input.as_bytes());

    // Decode input data
    let d = base64::engine::general_purpose::STANDARD_NO_PAD.decode(dr.input.as_bytes());
    let mut derr = String::new();
    let decoded = match d {
        Ok(v) => {
            use std::str;
            match str::from_utf8(&v) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    derr = "Decoded data is not a valid UTF-8 string".into();
                    hex::encode(&v)
                }
            }
        },
        Err(_) => {
            derr = "Invalid input to decode".into();
            String::new()
        },
    };

    cx.render(rsx!(
        div {
            class: "mt-2 row form-floating",
            textarea {
                id: "{TOOLID}-input",
                class: "form-control border-light",
                style: "height: 100px",
                oninput: move |evt| {
                    (*data.write()).input = evt.value.clone();
                },
                "{dr.input}"
            }
            label {
                class: "text-light",
                r#for: "{TOOLID}-input",
                "Input"
            }
        }

        div {
            class: "mt-2 row form-floating",
            textarea {
                id: "{TOOLID}-output-encoded",
                class: "form-control",
                style: "height: 100px",
                readonly: true,
                "{encoded}"
            }
            label {
                r#for: "{TOOLID}-output-encoded",
                "Encoded"
            }
        }

        div {
            class: "mt-2 row form-floating",
            textarea {
                id: "{TOOLID}-output-decoded",
                class: "form-control",
                style: "height: 100px",
                readonly: true,
                "{decoded}"
            }
            label {
                r#for: "{TOOLID}-output-decoded",
                "Decoded"
            }
            i {
                "{derr}"
            }
        }
    ))
}
