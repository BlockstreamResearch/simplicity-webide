//! DAG export utilities for visualizing Simplicity program structure.
//!
//! Matches the logic from hal-simplicity CLI's dag.rs for consistency.

use hex_conservative::DisplayHex;
use serde::{Deserialize, Serialize};
use simplicity::node::Inner;
use simplicityhl::simplicity;
use std::collections::HashMap;
use std::sync::Arc;

use crate::util::Expression;

/// Metadata for a single node in the DAG.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeMeta {
    pub id: String,
    pub kind: String,
    pub kind_class: String,
    pub type_arrow: String,
    pub desc: String,
    pub cmr: String,
    pub extra: Option<String>,
    pub children: Vec<String>,
}

/// An edge in the DAG (from parent to child for D3 tree layout).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

/// Complete DAG export for visualization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DagExport {
    pub root_id: String,
    pub nodes: Vec<NodeMeta>,
    pub edges: Vec<Edge>,
}

/// Build a DAG export from an Expression (RedeemNode).
pub fn build_dag_export(expr: &Arc<Expression>) -> DagExport {
    let mut builder = DagBuilder::new();
    let root_id = builder.visit(expr);
    
    DagExport {
        root_id,
        nodes: builder.nodes,
        edges: builder.edges,
    }
}

/// Builder that traverses the DAG recursively, matching CLI logic.
struct DagBuilder {
    /// Map from node pointer to assigned ID (for DAG sharing detection)
    node_ids: HashMap<usize, usize>,
    /// List of nodes in order of their IDs
    nodes: Vec<NodeMeta>,
    /// List of edges (parent → child for D3)
    edges: Vec<Edge>,
    /// Next available node ID
    next_id: usize,
}

impl DagBuilder {
    fn new() -> Self {
        Self {
            node_ids: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            next_id: 0,
        }
    }

    /// Visit a node and return its ID string.
    /// This matches the CLI's recursive approach: assign ID first, then process children.
    fn visit(&mut self, node: &Arc<Expression>) -> String {
        let ptr = Arc::as_ptr(node) as usize;

        // Check if we've already visited this node (DAG sharing)
        if let Some(&id) = self.node_ids.get(&ptr) {
            return format!("n{}", id);
        }

        // Assign ID in pre-order (before visiting children)
        let id = self.next_id;
        self.next_id += 1;
        self.node_ids.insert(ptr, id);
        let id_str = format!("n{}", id);

        // Get children and classify node
        let (kind, kind_class, desc, extra, children) = self.classify_node(node);

        // Process children and collect their IDs
        let child_ids: Vec<String> = children.iter().map(|c| self.visit(c)).collect();

        // Add edges from this node (parent) to its children
        // D3 tree layout expects parent → child edges
        for child_id in &child_ids {
            self.edges.push(Edge {
                from: id_str.clone(),
                to: child_id.clone(),
            });
        }

        // Get type arrow
        let arrow = node.arrow();
        let type_arrow = format!("{} → {}", arrow.source, arrow.target);

        // Get CMR as hex
        let cmr = node.cmr().as_ref().as_hex().to_string();

        self.nodes.push(NodeMeta {
            id: id_str.clone(),
            kind,
            kind_class,
            type_arrow,
            desc,
            cmr,
            extra,
            children: child_ids,
        });

        id_str
    }

    /// Classify a node and return (kind, kind_class, desc, extra, children).
    fn classify_node(&self, node: &Arc<Expression>) -> (String, String, String, Option<String>, Vec<Arc<Expression>>) {
        match node.inner() {
            Inner::Iden => (
                "iden".into(),
                "combinator".into(),
                "Identity: passes input through unchanged.".into(),
                None,
                vec![],
            ),
            Inner::Unit => (
                "unit".into(),
                "combinator".into(),
                "Produces unit value ().".into(),
                None,
                vec![],
            ),
            Inner::InjL(child) => (
                "injl".into(),
                "combinator".into(),
                "Inject value into left side of a sum type.".into(),
                None,
                vec![Arc::clone(child)],
            ),
            Inner::InjR(child) => (
                "injr".into(),
                "combinator".into(),
                "Inject value into right side of a sum type.".into(),
                None,
                vec![Arc::clone(child)],
            ),
            Inner::Take(child) => (
                "take".into(),
                "combinator".into(),
                "Project left element of a pair.".into(),
                None,
                vec![Arc::clone(child)],
            ),
            Inner::Drop(child) => (
                "drop".into(),
                "combinator".into(),
                "Project right element of a pair.".into(),
                None,
                vec![Arc::clone(child)],
            ),
            Inner::Comp(left, right) => (
                "comp".into(),
                "combinator".into(),
                "Composition: run left child then right child.".into(),
                None,
                vec![Arc::clone(left), Arc::clone(right)],
            ),
            Inner::Pair(left, right) => (
                "pair".into(),
                "combinator".into(),
                "Pairing: compute two functions on same input.".into(),
                None,
                vec![Arc::clone(left), Arc::clone(right)],
            ),
            Inner::Case(left, right) => (
                "case".into(),
                "combinator".into(),
                "Case split on Either type.".into(),
                None,
                vec![Arc::clone(left), Arc::clone(right)],
            ),
            Inner::AssertL(child, hidden_cmr) => {
                let cmr_hex = hidden_cmr.as_ref().as_hex().to_string();
                let short_cmr = if cmr_hex.len() > 8 { &cmr_hex[..8] } else { &cmr_hex };
                (
                    format!("assertl:{}", short_cmr),
                    "combinator".into(),
                    "Assert left branch.".into(),
                    Some(cmr_hex),
                    vec![Arc::clone(child)],
                )
            }
            Inner::AssertR(hidden_cmr, child) => {
                let cmr_hex = hidden_cmr.as_ref().as_hex().to_string();
                let short_cmr = if cmr_hex.len() > 8 { &cmr_hex[..8] } else { &cmr_hex };
                (
                    format!("assertr:{}", short_cmr),
                    "combinator".into(),
                    "Assert right branch.".into(),
                    Some(cmr_hex),
                    vec![Arc::clone(child)],
                )
            }
            Inner::Disconnect(left, _right) => (
                "disconnect".into(),
                "combinator".into(),
                "Disconnect combinator for delegation.".into(),
                None,
                vec![Arc::clone(left)],
            ),
            Inner::Witness(val) => {
                let val_str = format!("{:?}", val);
                (
                    "witness".into(),
                    "leaf".into(),
                    "Witness data placeholder.".into(),
                    Some(val_str),
                    vec![],
                )
            }
            Inner::Fail(entropy) => {
                let entropy_hex = entropy.as_ref().as_hex().to_string();
                let short = if entropy_hex.len() > 8 { &entropy_hex[..8] } else { &entropy_hex };
                (
                    format!("fail:{}", short),
                    "leaf".into(),
                    "Universal fail: always fails.".into(),
                    Some(entropy_hex),
                    vec![],
                )
            }
            Inner::Jet(jet) => {
                let jet_name = format!("{:?}", jet).to_lowercase();
                (
                    format!("jet_{}", jet_name),
                    "jet".into(),
                    format!("Jet primitive: {}", jet_name),
                    None,
                    vec![],
                )
            }
            Inner::Word(word) => {
                let word_str = word.as_value().to_string();
                let label = if word_str.len() > 16 {
                    format!("const:{}…", &word_str[..16])
                } else {
                    format!("const:{}", word_str)
                };
                (
                    label,
                    "leaf".into(),
                    format!("Constant word value: {}", word_str),
                    Some(word_str),
                    vec![],
                )
            }
        }
    }
}
