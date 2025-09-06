use std::{
    borrow::Borrow,
    collections::{
        HashMap,
        hash_map::{Keys, Values},
    },
    default,
    hash::Hash,
    ops::Index,
    slice::{Iter, SliceIndex},
};

use tiger_text::Language;

use crate::{
    data::{
        activity::{
            Activity, SActivityData, SActivityDisplayData, graph::SActivityGraph,
            types::SActivityType,
        },
        item::{InventoryItem, SInventoryItem, SInventoryItemDisplay},
    },
    global_instance::investment_manager,
};

pub mod data;
pub mod global_instance;
pub mod manager;

#[derive(Clone, strum::IntoStaticStr)]
pub enum InvestmentData {
    Achievement,
    Activity(Box<Activity>),
    ActivityGraph(Box<SActivityGraph>),
    ActivityType(Box<SActivityType>),
    InventoryItem(Box<InventoryItem>),
}

impl InvestmentData {
    pub fn name(&self) -> String {
        match self {
            Self::Activity(a) => a.display.display_properties.name.get().unwrap_or_default(),
            Self::InventoryItem(i) => i.display.name.get().unwrap_or_default(),
            // TODO: default missing name
            _ => String::new(),
        }
    }

    pub fn itype(&self) -> Option<String> {
        match self {
            Self::InventoryItem(i) => Some(i.display.item_type.get().unwrap_or_default()),
            // TODO: default missing name
            _ => None,
        }
    }

    pub fn hash(&self) -> u32 {
        match self {
            Self::Activity(a) => a.data.hash,
            Self::InventoryItem(i) => i.data.hash,
            _ => 0,
        }
    }
}

#[derive(Default)]
pub struct IndexableHashMap<K, V> {
    hash_to_index: HashMap<K, usize>,
    values: Vec<V>,
}

impl<K, V> IndexableHashMap<K, V> {
    pub fn get(&self, index: usize) -> Option<&V> {
        if index < self.values.len() && index < self.values.len() {
            Some(&self.values[index])
        } else {
            None
        }
    }

    pub fn keys(&self) -> Vec<&K> {
        self.hash_to_index.keys().collect()
    }

    pub fn values(&self) -> &Vec<V> {
        &self.values
    }

    pub fn new() -> Self {
        Self {
            hash_to_index: HashMap::new(),
            values: Vec::new(),
        }
    }
}

impl<K: Eq + Hash, V> IndexableHashMap<K, V> {
    pub fn lookup_hash(&self, hash: &K) -> Option<&V> {
        let index = self.hash_to_index.get(hash)?;
        self.values.get(*index)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<&V> {
        self.values.push(v);
        self.hash_to_index.insert(k, self.values.len() - 1);
        self.values.last()
    }
}
