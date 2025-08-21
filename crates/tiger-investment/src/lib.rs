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
            SActivityData, SActivityDisplayData, graph::SActivityGraph, types::SActivityType,
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
    Activity(Box<SActivityData>, Box<SActivityDisplayData>),
    ActivityGraph(Box<SActivityGraph>),
    ActivityType(Box<SActivityType>),
    InventoryItem(Box<InventoryItem>),
}

impl InvestmentData {
    pub fn name(&self, language: Language) -> String {
        match self {
            Self::Activity(_, display) => display
                .display_properties
                .name
                .get(language)
                .unwrap_or_default(),
            Self::InventoryItem(i) => i.display.name.get(language).unwrap_or_default(),
            // TODO: default missing name
            _ => String::new(),
        }
    }

    pub fn itype(&self, language: Language) -> Option<String> {
        match self {
            Self::InventoryItem(i) => Some(i.display.item_type.get(language).unwrap_or_default()),
            // TODO: default missing name
            _ => None,
        }
    }

    pub fn hash(&self) -> u32 {
        match self {
            Self::Activity(data, _) => data.hash,
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
    #[tracing::instrument(skip(self))]
    pub fn get(&self, index: usize) -> Option<&V> {
        if index < self.values.len() && index < self.values.len() {
            Some(&self.values[index])
        } else {
            None
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn keys(&self) -> Vec<&K> {
        self.hash_to_index.keys().collect()
    }

    #[tracing::instrument(skip(self))]
    pub fn values(&self) -> &Vec<V> {
        &self.values
    }

    #[tracing::instrument]
    pub fn new() -> Self {
        Self {
            hash_to_index: HashMap::new(),
            values: Vec::new(),
        }
    }
}

impl<K: Eq + Hash, V> IndexableHashMap<K, V> {
    #[tracing::instrument(skip_all)]
    pub fn lookup_hash(&self, hash: &K) -> Option<&V> {
        let index = self.hash_to_index.get(hash)?;
        self.values.get(*index)
    }

    #[tracing::instrument(skip_all)]
    pub fn insert(&mut self, k: K, v: V) -> Option<&V> {
        self.values.push(v);
        self.hash_to_index.insert(k, self.values.len() - 1);
        self.values.last()
    }
}
