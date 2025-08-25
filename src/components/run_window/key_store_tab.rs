use std::sync::Arc;

use elements::hashes::{sha256, Hash};
use elements::secp256k1_zkp as secp256k1;
use hex_conservative::{DisplayHex, FromHex};
use leptos::{
    component, create_memo, create_rw_signal, ev, event_target_value, html, use_context, view, For,
    IntoView, NodeRef, RwSignal, Signal, SignalGet, SignalGetUntracked, SignalSet, SignalUpdate,
    SignalWith, View,
};
use simplicityhl::{elements, simplicity};

use crate::components::copy_to_clipboard::CopyToClipboard;
use crate::util::{Counter26, SigningKeys};

#[derive(Copy, Clone, Debug, Default)]
pub struct KeyCount(pub RwSignal<Counter26>);

impl KeyCount {
    pub fn new(n: Counter26) -> Self {
        Self(create_rw_signal(n))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SignedDataMode {
    SighashAll,
    ThirtyTwoBytes,
    HashPreimageBytes,
}

#[derive(Clone, Copy, Debug)]
pub struct SignedData {
    pub mode: RwSignal<SignedDataMode>,
    pub thirty_two_bytes: RwSignal<[u8; 32]>,
    pub sighash_all: Signal<secp256k1::Message>,
    pub hash_preimage_bytes: RwSignal<Vec<u8>>,
    pub message: Signal<secp256k1::Message>,
}

impl SignedData {
    pub fn new(
        tx_env: Signal<simplicity::jet::elements::ElementsEnv<Arc<elements::Transaction>>>,
    ) -> Self {
        let mode = create_rw_signal(SignedDataMode::SighashAll);
        let sighash_all = Signal::derive(move || {
            tx_env.with(|tx_env| {
                secp256k1::Message::from_digest(tx_env.c_tx_env().sighash_all().to_byte_array())
            })
        });
        let thirty_two_bytes = create_rw_signal([0; 32]);
        let hash_preimage_bytes = create_rw_signal(vec![]);
        let message = Signal::derive(move || match mode.get() {
            SignedDataMode::SighashAll => sighash_all.get(),
            SignedDataMode::ThirtyTwoBytes => {
                secp256k1::Message::from_digest(thirty_two_bytes.get())
            }
            SignedDataMode::HashPreimageBytes => hash_preimage_bytes.with(|bytes| {
                secp256k1::Message::from_digest(sha256::Hash::hash(bytes).to_byte_array())
            }),
        });
        Self {
            mode,
            thirty_two_bytes,
            sighash_all,
            hash_preimage_bytes,
            message,
        }
    }
}

fn key_name(index: usize) -> &'static str {
    match index {
        0 => "Alice",
        1 => "Bob",
        2 => "Charlie",
        3 => "David",
        4 => "Eve",
        5 => "Frank",
        6 => "Grace",
        7 => "Heidi",
        8 => "Ivan",
        9 => "Judy",
        10 => "Kevin",
        11 => "Luther",
        12 => "Mallory",
        13 => "Niaj",
        14 => "Olivia",
        15 => "Peggy",
        16 => "Quentin",
        17 => "Rupert",
        18 => "Sybil",
        19 => "Trent",
        20 => "Ursula",
        21 => "Victor",
        22 => "Wendy",
        23 => "Xavier",
        24 => "Yvonne",
        25 => "Zoe",
        _ => "Unnamed",
    }
}

#[component]
pub fn KeyStoreTab() -> impl IntoView {
    view! {
        <div class="tab-content key-store-tab">
            <CopyPublicKeysToClipboard />
            <CopySignaturesToClipboard />
            <SelectSignedData />
        </div>
    }
}

#[component]
fn CopyPublicKeysToClipboard() -> impl IntoView {
    let signing_keys = use_context::<SigningKeys>().expect("signing keys should exist in context");
    let key_count = use_context::<KeyCount>().expect("key count should exist in context");
    let copy_single_public_key = move |index: usize| -> View {
        let label = key_name(index);
        let xonly_hex =
            move || format!("0x{}", signing_keys.public_keys[index].serialize().as_hex());

        view! {
            <CopyToClipboard content=xonly_hex class="copy-button">
                {label}
                <i class="far fa-copy"></i>
            </CopyToClipboard>
        }
    };

    view! {
        <div>
            <p class="tab-description">
                "The secret master key is stored in the browser's local storage. Anyone with access to this key can sweep your coins."
            </p>

            <div class="tab-title-group">
                <h3 class="tab-title">
                    Public Keys
                </h3>

                <div class="button-row is-small">
                    <button
                        class="flat-button bordered"
                        type="button"
                        on:click=move |_| key_count.0.update(Counter26::saturating_increment)
                    >
                        <i class="fas fa-plus"></i>
                        More
                    </button>
                    <button
                        class="flat-button bordered"
                        type="button"
                        on:click=move |_| key_count.0.update(Counter26::saturating_decrement)
                    >
                        <i class="fas fa-minus"></i>
                        Less
                    </button>
                </div>
            </div>
            <div class="button-row is-small">
                <For
                    each=move || 0..key_count.0.get().get()
                    key=|index| *index
                    children=copy_single_public_key
                />
            </div>
        </div>
    }
}

#[component]
fn CopySignaturesToClipboard() -> impl IntoView {
    let signing_keys = use_context::<SigningKeys>().expect("signing keys should exist in context");
    let signed_data = use_context::<SignedData>().expect("signed data should exist in context");
    let key_count = use_context::<KeyCount>().expect("key count should exist in context");
    let signatures = create_memo(move |_| -> [secp256k1::schnorr::Signature; 26] {
        std::array::from_fn(|index| {
            signing_keys.secret_keys[index].sign_schnorr(signed_data.message.get())
        })
    });

    let copy_single_signature =
        move |(index, signature): (usize, secp256k1::schnorr::Signature)| -> View {
            let label = key_name(index);
            let signature_hex = move || format!("0x{}", signature.serialize().as_hex());

            view! {
                <CopyToClipboard content=signature_hex class="copy-button">
                    {label}
                    <i class="far fa-copy"></i>
                </CopyToClipboard>
            }
        };

    view! {
        <div>
            <div class="tab-title-group">
                <h3 class="tab-title">
                    Signatures
                </h3>

                <div class="button-row is-small">
                    <button
                        class="flat-button bordered"
                        type="button"
                        on:click=move |_| key_count.0.update(Counter26::saturating_increment)
                    >
                        <i class="fas fa-plus"></i>
                        More
                    </button>
                    <button
                        class="flat-button bordered"
                        type="button"
                        on:click=move |_| key_count.0.update(Counter26::saturating_increment)
                    >
                        <i class="fas fa-minus"></i>
                        Less
                    </button>
                </div>
            </div>

            <div class="button-row is-small">
                <For
                    each=move || (0..key_count.0.get().get()).zip(signatures.get().into_iter())
                    key=|(_index, signature)| *signature
                    children=copy_single_signature
                />
            </div>
        </div>
    }
}

#[component]
fn SelectSignedData() -> impl IntoView {
    let signed_data = use_context::<SignedData>().expect("signed data should exist in context");
    let thirty_two_bytes_is_insane = create_rw_signal(false);
    let hash_preimage_bytes_is_insane = create_rw_signal(false);

    let sighash_all_initial_checked =
        matches!(signed_data.mode.get_untracked(), SignedDataMode::SighashAll);
    let thirty_two_bytes_initial_checked = matches!(
        signed_data.mode.get_untracked(),
        SignedDataMode::ThirtyTwoBytes
    );
    let hash_preimage_bytes_initial_checked = matches!(
        signed_data.mode.get_untracked(),
        SignedDataMode::HashPreimageBytes
    );
    let thirty_two_bytes_initial_value = format!(
        "0x{}",
        signed_data.thirty_two_bytes.get_untracked().as_hex()
    );
    let hash_preimage_bytes_initial_value = format!(
        "0x{}",
        signed_data.hash_preimage_bytes.get_untracked().as_hex()
    );

    let sighash_all_radio_ref = NodeRef::<html::Input>::new();
    let thirty_two_bytes_text_ref = NodeRef::<html::Input>::new();
    let hash_preimage_bytes_text_ref = NodeRef::<html::Input>::new();

    let select_sighash_all = move |_event: ev::Event| {
        signed_data.mode.set(SignedDataMode::SighashAll);
    };
    let select_thirty_two_bytes = move |_event: ev::Event| {
        signed_data.mode.set(SignedDataMode::ThirtyTwoBytes);
    };
    let select_hash_preimage_bytes = move |_event: ev::Event| {
        signed_data.mode.set(SignedDataMode::HashPreimageBytes);
    };
    let update_thirty_two_bytes = move |event: ev::Event| match <[u8; 32]>::from_hex(
        event_target_value(&event)
            .as_str()
            .trim()
            .trim_start_matches("0x"),
    ) {
        Ok(bytes) => {
            signed_data.thirty_two_bytes.set(bytes);
            thirty_two_bytes_text_ref
                .get()
                .expect("<input> should be mounted")
                .set_custom_validity("");
            thirty_two_bytes_is_insane.set(false);
        }
        Err(..) => {
            sighash_all_radio_ref
                .get()
                .expect("<input> should be mounted")
                .set_checked(true);
            thirty_two_bytes_text_ref
                .get()
                .expect("<input> should be mounted")
                .set_custom_validity("Expected exactly 64 hex digits");
            thirty_two_bytes_is_insane.set(true);
        }
    };
    let update_hash_preimage_bytes = move |event: ev::Event| {
        if let Ok(bytes) = <Vec<u8>>::from_hex(
            event_target_value(&event)
                .as_str()
                .trim()
                .trim_start_matches("0x"),
        ) {
            signed_data.hash_preimage_bytes.set(bytes);
            hash_preimage_bytes_text_ref
                .get()
                .expect("<input> should be mounted")
                .set_custom_validity("");
            hash_preimage_bytes_is_insane.set(false);
        } else {
            sighash_all_radio_ref
                .get()
                .expect("<input> should be mounted")
                .set_checked(true);
            hash_preimage_bytes_text_ref
                .get()
                .expect("<input> should be mounted")
                .set_custom_validity("Expected even number of hex digits");
            hash_preimage_bytes_is_insane.set(true);
        }
    };

    view! {
        <div>
            <h3 class="tab-title">
                Signed Data
            </h3>
            <fieldset class="signed-data-content">
                <label class="key-store-display-row">
                    <input
                        type="radio"
                        name="signed_data"
                        checked=sighash_all_initial_checked
                        on:change=select_sighash_all
                        node_ref=sighash_all_radio_ref
                    />
                    <div class="display-row-label">
                        SIGHASH_ALL
                    </div>
                </label>
                <label class="key-store-display-row">
                    <input
                        type="radio"
                        name="signed_data"
                        checked=thirty_two_bytes_initial_checked
                        on:change=select_thirty_two_bytes
                        disabled=thirty_two_bytes_is_insane
                    />
                    <div class="display-row-label">
                        raw bytes
                    </div>
                    <input
                        class="input"
                        type="text"
                        placeholder="Enter hex bytes"
                        on:input=update_thirty_two_bytes
                        value=thirty_two_bytes_initial_value
                        node_ref=thirty_two_bytes_text_ref
                    />
                </label>
                <label class="key-store-display-row">
                    <input
                        type="radio"
                        name="signed_data"
                        checked=hash_preimage_bytes_initial_checked
                        on:change=select_hash_preimage_bytes
                        disabled=hash_preimage_bytes_is_insane
                    />
                    <div class="display-row-label">
                        byte hash
                    </div>
                    <input
                        class="input"
                        type="text"
                        placeholder="Enter hex bytes"
                        on:input=update_hash_preimage_bytes
                        value=hash_preimage_bytes_initial_value
                        node_ref=hash_preimage_bytes_text_ref
                    />
                </label>
            </fieldset>
        </div>
    }
}
