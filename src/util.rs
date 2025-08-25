use elements::hashes::{sha256, Hash};
use elements::secp256k1_zkp as secp256k1;
use secp256k1::rand::{self, Rng, SeedableRng};
use simplicity::dag::{DagLike, MaxSharing, NoSharing};
use simplicity::jet::Elements;
use simplicity::{node, RedeemNode};
use simplicityhl::num::U256;
use simplicityhl::simplicity::Preimage32;
use simplicityhl::{elements, simplicity};

#[derive(Clone, Debug)]
pub struct SigningKeys {
    pub random_seed: U256,
    pub secret_keys: [secp256k1::Keypair; 26],
    pub public_keys: [secp256k1::XOnlyPublicKey; 26],
}

impl SigningKeys {
    pub fn new(random_seed: U256) -> Self {
        let mut rng = rand::rngs::StdRng::from_seed(random_seed.to_byte_array());
        let secret_keys =
            std::array::from_fn(|_| secp256k1::Keypair::new(secp256k1::SECP256K1, &mut rng));
        let public_keys = std::array::from_fn(|index| secret_keys[index].x_only_public_key().0);
        Self {
            random_seed,
            secret_keys,
            public_keys,
        }
    }
}

impl Default for SigningKeys {
    fn default() -> Self {
        Self::new(U256::MIN)
    }
}

#[derive(Clone, Debug)]
pub struct HashedData {
    pub random_seed: U256,
    pub preimages: [Preimage32; 26],
    pub hashes: [sha256::Hash; 26],
}

impl HashedData {
    pub fn new(random_seed: U256) -> Self {
        let mut rng = rand::rngs::StdRng::from_seed(random_seed.to_byte_array());
        let preimages = std::array::from_fn(|_| {
            let mut preimage = [0; 32];
            rng.fill(&mut preimage);
            preimage
        });
        let hashes = std::array::from_fn(|index| sha256::Hash::hash(&preimages[index]));
        Self {
            random_seed,
            preimages,
            hashes,
        }
    }
}

impl Default for HashedData {
    fn default() -> Self {
        Self::new(U256::MIN)
    }
}

/// A counter in the range `1..26`.
#[derive(Copy, Clone, Debug)]
pub struct Counter26(usize);

impl Default for Counter26 {
    fn default() -> Self {
        Self(1)
    }
}

impl Counter26 {
    pub fn new(n: usize) -> Option<Self> {
        match 0 < n && n < 26 {
            true => Some(Self(n)),
            false => None,
        }
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn saturating_increment(&mut self) {
        if self.0 < 26 {
            self.0 = self.0.saturating_add(1);
        }
        debug_assert!(0 < self.0 && self.0 < 26);
    }

    pub fn saturating_decrement(&mut self) {
        if 0 < self.0 - 1 {
            self.0 -= 1;
        }
        debug_assert!(0 < self.0 && self.0 < 26);
    }
}

pub type Expression = RedeemNode<Elements>;

pub fn get_compression_factor<M: node::Marker>(node: &node::Node<M>) -> usize {
    let unshared_len = node.pre_order_iter::<NoSharing>().count();
    let shared_len = node.pre_order_iter::<MaxSharing<M>>().count();
    debug_assert!(0 < shared_len);
    debug_assert!(shared_len <= unshared_len);
    unshared_len / shared_len
}

fn unspendable_internal_key() -> secp256k1::XOnlyPublicKey {
    secp256k1::XOnlyPublicKey::from_slice(&[
        0xf5, 0x91, 0x9f, 0xa6, 0x4c, 0xe4, 0x5f, 0x83, 0x06, 0x84, 0x90, 0x72, 0xb2, 0x6c, 0x1b,
        0xfd, 0xd2, 0x93, 0x7e, 0x6b, 0x81, 0x77, 0x47, 0x96, 0xff, 0x37, 0x2b, 0xd1, 0xeb, 0x53,
        0x62, 0xd2,
    ])
    .expect("key should be valid")
}

fn script_ver(cmr: simplicity::Cmr) -> (elements::Script, elements::taproot::LeafVersion) {
    let script = elements::script::Script::from(cmr.as_ref().to_vec());
    (script, simplicity::leaf_version())
}

fn taproot_spend_info(cmr: simplicity::Cmr) -> elements::taproot::TaprootSpendInfo {
    let builder = elements::taproot::TaprootBuilder::new();
    let (script, version) = script_ver(cmr);
    let builder = builder
        .add_leaf_with_ver(0, script, version)
        .expect("tap tree should be valid");
    builder
        .finalize(secp256k1::SECP256K1, unspendable_internal_key())
        .expect("tap tree should be valid")
}

pub fn liquid_testnet_address(cmr: simplicity::Cmr) -> elements::Address {
    let info = taproot_spend_info(cmr);
    let blinder = None;
    elements::Address::p2tr(
        secp256k1::SECP256K1,
        info.internal_key(),
        info.merkle_root(),
        blinder,
        &elements::AddressParams::LIQUID_TESTNET,
    )
}

pub fn liquid_testnet_bitcoin_asset() -> elements::AssetId {
    elements::AssetId::from_inner(sha256::Midstate([
        0x49, 0x9a, 0x81, 0x85, 0x45, 0xf6, 0xba, 0xe3, 0x9f, 0xc0, 0x3b, 0x63, 0x7f, 0x2a, 0x4e,
        0x1e, 0x64, 0xe5, 0x90, 0xca, 0xc1, 0xbc, 0x3a, 0x6f, 0x6d, 0x71, 0xaa, 0x44, 0x43, 0x65,
        0x4c, 0x14,
    ]))
}

pub fn liquid_testnet_genesis() -> elements::BlockHash {
    elements::BlockHash::from_byte_array([
        0xc1, 0xb1, 0x6a, 0xe2, 0x4f, 0x24, 0x23, 0xae, 0xa2, 0xea, 0x34, 0x55, 0x22, 0x92, 0x79,
        0x3b, 0x5b, 0x5e, 0x82, 0x99, 0x9a, 0x1e, 0xed, 0x81, 0xd5, 0x6a, 0xee, 0x52, 0x8e, 0xda,
        0x71, 0xa7,
    ])
}

pub fn liquid_testnet_faucet_script_pubkey() -> elements::Script {
    "tlq1qqd0qxdqsag3t63gfzq4xr25fcjvsujun6ycx9jtd9jufarrrwtseyf05kf0qz62u09wpnj064cycfvtlxuz4xj4j48wxpsrs2"
        .parse::<elements::Address>()
        .expect("address should be valid").script_pubkey()
}

pub fn control_block(cmr: simplicity::Cmr) -> elements::taproot::ControlBlock {
    let info = taproot_spend_info(cmr);
    let script_ver = script_ver(cmr);
    info.control_block(&script_ver)
        .expect("control block should exist")
}
