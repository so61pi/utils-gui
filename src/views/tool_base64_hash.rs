use dioxus::prelude::*;
use sha1::Digest;
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
enum HashInputKind {
    #[strum(to_string = "UTF-8")]
    Utf8,
    Hex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolHashComponentData {
    input: String,
    input_kind: HashInputKind,
}

impl Default for ToolHashComponentData {
    fn default() -> Self {
        Self {
            input: Default::default(),
            input_kind: HashInputKind::Utf8,
        }
    }
}

#[component]
pub fn ToolHashComponent(
    data: ToolHashComponentData,
    onupdate: EventHandler<ToolHashComponentData>,
) -> Element {
    let input = match data.input_kind {
        HashInputKind::Utf8 => Ok(data.input.as_bytes().to_vec()),
        HashInputKind::Hex => {
            let input = data.input.replace([' ', '\t'], "");
            hex::decode(&input)
        }
    };

    let hashers: Vec<(_, Box<dyn digest::DynDigest>)> = vec![
        ("MD5", Box::new(md5::Md5::new())),
        ("SHA1", Box::new(sha1::Sha1::new())),
        ("SHA2-224", Box::new(sha2::Sha224::new())),
        ("SHA2-256", Box::new(sha2::Sha256::new())),
        ("SHA2-384", Box::new(sha2::Sha384::new())),
        ("SHA2-512", Box::new(sha2::Sha512::new())),
        ("SHA2-512/224", Box::new(sha2::Sha512_224::new())),
        ("SHA2-512/256", Box::new(sha2::Sha512_256::new())),
        ("SHA3-224", Box::new(sha3::Sha3_224::new())),
        ("SHA3-256", Box::new(sha3::Sha3_256::new())),
        ("SHA3-384", Box::new(sha3::Sha3_384::new())),
        ("SHA3-512", Box::new(sha3::Sha3_512::new())),
    ];
    let hashe_rows = hashers.into_iter().map(|(name, mut hasher)| {
        let hash_size_bytes = hasher.output_size();
        let hash = input
            .as_ref()
            .map(|v| {
                hasher.update(v);
                hex::encode(hasher.finalize())
            })
            .unwrap_or_default();

        rsx! {
            tr {
                th { scope: "row", "{name}" }
                td { "{hash_size_bytes} ({hash_size_bytes*8})" }
                td { "{hash}" }
            }
        }
    });
    let input_valid = input.is_ok();

    let input_kinds = HashInputKind::iter().map(|v| {
        let id = format!(
            "ToolHashComponent-btn-radio-input-{}",
            HashInputKindDiscriminants::from(v)
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
                    onupdate.call(ToolHashComponentData{
                        input_kind: v,
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
                        onupdate.call(ToolHashComponentData{
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
            }
            table { class: "table table-hover font-monospace selectable",
                thead {
                    tr {
                        th { scope: "col", "Algorithm" }
                        th { scope: "col", "Bytes (Bits)" }
                        th { scope: "col", "Hash" }
                    }
                }
                tbody {
                    { hashe_rows }
                }
            }
        }
    }
}
