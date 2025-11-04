use std::sync::Arc;

use elements::confidential;
use elements::hashes::Hash;
use simplicity::jet::elements::{ElementsEnv, ElementsUtxo};
use simplicityhl::simplicity::jet::Elements;
use simplicityhl::simplicity::RedeemNode;
use simplicityhl::{elements, simplicity};

use crate::util;

#[derive(Clone, Debug)]
pub struct TxParams {
    pub txid: elements::Txid,
    pub vout: u32,
    pub value_in: u64,
    pub recipient_address: Option<elements::Address>,
    pub fee: u64,
    pub lock_time: elements::LockTime,
    pub sequence: elements::Sequence,
}

impl Default for TxParams {
    fn default() -> Self {
        Self {
            txid: elements::Txid::all_zeros(),
            vout: 0,
            value_in: 100_000,
            recipient_address: None,
            fee: 1_000,
            lock_time: elements::LockTime::from_consensus(0),
            sequence: elements::Sequence::from_consensus(0),
        }
    }
}

impl TxParams {
    fn unsatisfied_transaction(&self) -> elements::Transaction {
        elements::Transaction {
            version: 2,
            lock_time: self.lock_time,
            input: vec![elements::TxIn {
                previous_output: elements::OutPoint {
                    txid: self.txid,
                    vout: self.vout,
                },
                is_pegin: false,
                script_sig: elements::Script::new(),
                sequence: self.sequence,
                asset_issuance: elements::AssetIssuance::null(),
                witness: elements::TxInWitness::empty(), // not required here
            }],
            output: vec![
                elements::TxOut {
                    asset: confidential::Asset::Explicit(util::liquid_testnet_bitcoin_asset()),
                    value: confidential::Value::Explicit(self.value_in.saturating_sub(self.fee)),
                    nonce: confidential::Nonce::Null,
                    script_pubkey: self.recipient_address.as_ref().map_or_else(
                        util::liquid_testnet_faucet_script_pubkey,
                        elements::Address::script_pubkey,
                    ),
                    witness: elements::TxOutWitness::empty(),
                },
                elements::TxOut::new_fee(self.fee, util::liquid_testnet_bitcoin_asset()),
            ],
        }
    }

    fn utxo(&self, script_pubkey: elements::Script) -> ElementsUtxo {
        ElementsUtxo {
            script_pubkey,
            asset: confidential::Asset::Explicit(util::liquid_testnet_bitcoin_asset()),
            value: confidential::Value::Explicit(self.value_in),
        }
    }

    pub fn tx_env(&self, cmr: simplicity::Cmr) -> ElementsEnv<Arc<elements::Transaction>> {
        let script_pubkey = util::liquid_testnet_address(cmr).script_pubkey();
        let index = 0;
        let annex = None;
        ElementsEnv::new(
            Arc::new(self.unsatisfied_transaction()),
            vec![self.utxo(script_pubkey)],
            index,
            cmr,
            util::control_block(cmr),
            annex,
            util::liquid_testnet_genesis(),
        )
    }

    pub fn transaction(&self, pruned: &RedeemNode<Elements>) -> elements::Transaction {
        let mut tx = self.unsatisfied_transaction();
        let (simplicity_program_bytes, simplicity_witness_bytes) = pruned.to_vec_with_witness();
        let cmr = pruned.cmr();
        tx.input[0].witness = elements::TxInWitness {
            amount_rangeproof: None,
            inflation_keys_rangeproof: None,
            script_witness: vec![
                simplicity_witness_bytes,
                simplicity_program_bytes,
                cmr.as_ref().to_vec(),
                util::control_block(cmr).serialize(),
            ],
            pegin_witness: vec![],
        };
        tx
    }
}
