use dioxus::prelude::*;
use digest::Digest;

const TOOLID: &str = "tool-hash";

#[derive(Default)]
pub struct ToolData {
    input: String,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::Display)]
pub enum HashAlgo {
    SHA1,

    // SHA-2
    SHA224,
    SHA256,
    SHA512_224,
    SHA512_256,
    SHA384,

    // SHA-3
    SHA3_224,
    SHA3_256,
    SHA3_384,
    SHA3_512,
}

#[inline_props]
pub fn ToolHash(cx: Scope, algo: HashAlgo) -> Element {
    let data = use_shared_state::<ToolData>(cx).unwrap();
    let dr = data.read();

    let toolid = format!("{TOOLID}-{algo}");

    let hashed = hex::encode(hash(algo.clone(), dr.input.as_bytes()));
    cx.render(rsx!(
        div {
            class: "mt-2 row form-floating",
            textarea {
                id: "{toolid}-input",
                class: "form-control border-light",
                style: "height: 100px",
                oninput: move |evt| {
                    (*data.write()).input = evt.value.clone();
                },
                "{dr.input}"
            }
            label {
                class: "text-light",
                r#for: "{toolid}-input",
                "Input"
            }
        }

        div {
            class: "mt-2 row form-floating",
            textarea {
                id: "{toolid}-output",
                class: "form-control",
                style: "height: 100px",
                readonly: true,
                "{hashed}"
            }
            label {
                r#for: "{toolid}-output",
                "Hash"
            }
        }
    ))
}

fn hash(algo: HashAlgo, input: &[u8]) -> Vec<u8> {
    let mut hasher: Box<dyn digest::DynDigest> = match algo {
        HashAlgo::SHA1 => Box::new(sha1::Sha1::new()),
        HashAlgo::SHA224 => Box::new(sha2::Sha224::new()),
        HashAlgo::SHA256 => Box::new(sha2::Sha256::new()),
        HashAlgo::SHA512_224 => Box::new(sha2::Sha512_224::new()),
        HashAlgo::SHA512_256 => Box::new(sha2::Sha512_256::new()),
        HashAlgo::SHA384 => Box::new(sha2::Sha384::new()),
        HashAlgo::SHA3_224 => Box::new(sha3::Sha3_224::new()),
        HashAlgo::SHA3_256 => Box::new(sha3::Sha3_256::new()),
        HashAlgo::SHA3_384 => Box::new(sha3::Sha3_384::new()),
        HashAlgo::SHA3_512 => Box::new(sha3::Sha3_512::new()),
    };

    hasher.update(input);
    hasher.finalize().to_vec()
}
