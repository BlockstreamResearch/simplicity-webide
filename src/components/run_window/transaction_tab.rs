use std::str::FromStr;
use std::sync::Arc;

use leptos::{
    component, create_rw_signal, ev, event_target_value, use_context, view, with, Children,
    IntoView, RwSignal, Signal, SignalGetUntracked, SignalSet, SignalUpdate,
};
use simplicity::jet::elements::ElementsEnv;
use simplicityhl::{elements, simplicity};

use crate::components::program_window::Program;
use crate::components::string_box::ErrorBox;
use crate::transaction::TxParams;

#[derive(Copy, Clone, Debug)]
pub struct TxEnv {
    pub params: RwSignal<TxParams>,
    pub lazy_env: Signal<ElementsEnv<Arc<elements::Transaction>>>,
}

impl TxEnv {
    pub fn new(program: Program, params: TxParams) -> Self {
        let params = create_rw_signal(params);
        let lazy_cmr = program.lazy_cmr;
        let lazy_env = Signal::derive(move || {
            with!(|params, lazy_cmr| match lazy_cmr {
                Ok(cmr) => params.tx_env(*cmr),
                Err(..) => params.tx_env(simplicity::Cmr::unit()),
            })
        });
        Self { params, lazy_env }
    }
}

#[component]
pub fn TransactionTab() -> impl IntoView {
    let tx_env = use_context::<TxEnv>().expect("transaction environment should exist in context");
    let txid_parse_error = create_rw_signal(String::new());
    let vout_parse_error = create_rw_signal(String::new());
    let value_in_parse_error = create_rw_signal(String::new());
    let recipient_address_parse_error = create_rw_signal(String::new());
    let fee_parse_error = create_rw_signal(String::new());
    let lock_time_parse_error = create_rw_signal(String::new());
    let sequence_parse_error = create_rw_signal(String::new());

    let update_txid = move |e: ev::Event| match elements::Txid::from_str(&event_target_value(&e)) {
        Ok(txid) => {
            tx_env.params.update(|x| x.txid = txid);
            txid_parse_error.update(String::clear);
        }
        Err(error) => txid_parse_error.set(error.to_string()),
    };
    let update_vout = move |e: ev::Event| match event_target_value(&e).parse::<u32>() {
        Ok(vout) => {
            tx_env.params.update(|x| x.vout = vout);
            vout_parse_error.update(String::clear);
        }
        Err(error) => vout_parse_error.set(error.to_string()),
    };
    let update_value_in = move |e: ev::Event| match event_target_value(&e).parse::<u64>() {
        Ok(value_in) => {
            tx_env.params.update(|x| x.value_in = value_in);
            value_in_parse_error.update(String::clear);
        }
        Err(error) => value_in_parse_error.set(error.to_string()),
    };
    let update_recipient_address = move |e: ev::Event| {
        let s = event_target_value(&e);
        match elements::Address::parse_with_params(&s, &elements::AddressParams::LIQUID_TESTNET) {
            Ok(address) => {
                tx_env
                    .params
                    .update(|x| x.recipient_address = Some(address));
                recipient_address_parse_error.update(String::clear);
            }
            Err(..) if s.is_empty() => {
                tx_env.params.update(|x| x.recipient_address = None);
                recipient_address_parse_error.update(String::clear);
            }
            Err(error) => recipient_address_parse_error.set(error.to_string()),
        }
    };
    let update_fee = move |e: ev::Event| match event_target_value(&e).parse::<u64>() {
        Ok(fee) => {
            tx_env.params.update(|x| x.fee = fee);
            fee_parse_error.update(String::clear);
        }
        Err(error) => fee_parse_error.set(error.to_string()),
    };
    let update_lock_time = move |e: ev::Event| match event_target_value(&e).parse::<u32>() {
        Ok(lock_time) => {
            let lock_time = elements::LockTime::from_consensus(lock_time);
            tx_env.params.update(|x| x.lock_time = lock_time);
            lock_time_parse_error.update(String::clear);
        }
        Err(error) => lock_time_parse_error.set(error.to_string()),
    };
    let update_sequence = move |e: ev::Event| match event_target_value(&e).parse::<u32>() {
        Ok(sequence) => {
            let sequence = elements::Sequence::from_consensus(sequence);
            tx_env.params.update(|x| x.sequence = sequence);
            sequence_parse_error.update(String::clear);
        }
        Err(error) => sequence_parse_error.set(error.to_string()),
    };

    view! {
        <div class="tab-content transaction-tab">
            <p class="tab-description">
                "Only a limited number of fields are available. "
                "More customization will follow in future updates."
            </p>
            <Section name="UTXO">
                <Item name="txid" error=txid_parse_error>
                    <input
                        class="input"
                        type="text"
                        on:input=update_txid
                        value=tx_env.params.get_untracked().txid.to_string()
                    />
                </Item>
                <Item name="vout" error=vout_parse_error>
                    <input
                        class="input"
                        type="number"
                        min=0
                        on:input=update_vout
                        value=tx_env.params.get_untracked().vout
                    />
                </Item>
                <Item name="value (sats)" error=value_in_parse_error>
                    <input
                        class="input"
                        type="number"
                        min=0
                        on:input=update_value_in
                        value=tx_env.params.get_untracked().value_in
                    />
                </Item>
            </Section>
            <Section name="Transaction">
                <Item name="recipient address" error=recipient_address_parse_error>
                    <input
                        class="input"
                        type="text"
                        on:input=update_recipient_address
                        value=tx_env.params.get_untracked().recipient_address.as_ref().map(ToString::to_string).unwrap_or_default()
                        placeholder="(Send back to faucet)"
                    />
                </Item>
                <Item name="fee (sats)" error=fee_parse_error>
                    <input
                        class="input"
                        type="number"
                        on:input=update_fee
                        min=0
                        value=10000
                        value=tx_env.params.get_untracked().fee
                    />
                </Item>
                <Item name="nLockTime" error=lock_time_parse_error>
                    <input
                        class="input"
                        type="number"
                        on:input=update_lock_time
                        min=0
                        value=tx_env.params.get_untracked().lock_time.to_string()
                    />
                </Item>
                <Item name="nSequence" error=sequence_parse_error>
                    <input
                        class="input"
                        type="number"
                        on:input=update_sequence
                        min=0
                        value=tx_env.params.get_untracked().sequence.to_string()
                    />
                </Item>
            </Section>
        </div>
    }
}

#[component]
fn Section(#[prop(into)] name: String, children: Children) -> impl IntoView {
    view! {
        <h3 class="tab-title">
            {name}
        </h3>
        <div>
            {children()}
        </div>
    }
}

#[component]
fn Item(
    #[prop(into)] name: String,
    #[prop(into)] error: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="transaction-display-row">
            <div class="display-row-label">
                {name}
            </div>
            {children()}
        </div>
        <ErrorBox error=error />
    }
}
