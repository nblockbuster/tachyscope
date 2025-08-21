use std::{collections::BTreeMap, sync::Arc};

use crossbeam::channel::TrySendError;
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;
use tiger_parse::{PackageManagerExt, TigerReadable};
use tiger_pkg::package_manager;
use tiger_text::{Language, LocalizedStrings};

use crate::{
    IndexableHashMap, InvestmentData,
    data::{
        activity::{SActivityData, SActivityDisplayData, SActivityDisplayList, SActivityList},
        image::{SInvestmentIcon, SInvestmentIcons},
        item::{InventoryItem, SInventoryItem, SInventoryItemDisplayList, SItemList},
        text::SIndexedLocalizedStrings,
    },
};

pub struct InvestmentManager {
    activities: Arc<ActivityManager>,
    strings: Arc<StringManager>,
    items: Arc<ItemManager>,
    icons: Arc<IconManager>,
}

pub struct ActivityManager {
    activities: IndexableHashMap<u32, InvestmentData>,
}

pub struct StringManager {
    string_cache: RwLock<FxHashMap<(u32, u32), String>>,
    indexed_strings: SIndexedLocalizedStrings,
}

pub struct ItemManager {
    items: IndexableHashMap<u32, InvestmentData>,
}

pub struct IconManager {
    // TODO: IndexableHashMap? They have hashes, but don't know if used elsewhere, and not just by indexing
    icon_cache: RwLock<FxHashMap<usize, SInvestmentIcon>>,
    icon_tag: SInvestmentIcons,
}

impl InvestmentManager {
    #[tracing::instrument]
    pub fn new() -> anyhow::Result<Self> {
        let activity_display_list: SActivityDisplayList = package_manager().read_tag_struct(
            package_manager().get_all_by_reference(SActivityDisplayList::ID.unwrap())[0].0,
        )?;
        let activities_data: SActivityList = package_manager().read_tag_struct(
            package_manager().get_all_by_reference(SActivityList::ID.unwrap())[0].0,
        )?;

        let mut activities = IndexableHashMap::new();

        if activity_display_list.activities.len() != activities_data.activities.len() {
            return Err(anyhow::anyhow!("Activity table size does not match"));
        }

        for (i, a_data) in activities_data.activities.iter().enumerate() {
            let a_disp = &activity_display_list.activities[i];
            activities.insert(
                a_data.hash,
                InvestmentData::Activity(
                    Box::new(a_data.activity.0.clone()),
                    Box::new(a_disp.activity.0.clone()),
                ),
            );
        }

        let item_tag: SItemList = package_manager()
            .read_tag_struct(package_manager().get_all_by_reference(SItemList::ID.unwrap())[0].0)?;

        let item_display_tag: SInventoryItemDisplayList = package_manager().read_tag_struct(
            package_manager().get_all_by_reference(SInventoryItemDisplayList::ID.unwrap())[0].0,
        )?;

        let mut item_map = IndexableHashMap::new();

        for (i, data) in item_tag.items.iter().enumerate() {
            let item_display = &item_display_tag.stringmap[i];
            item_map.insert(
                data.hash,
                InvestmentData::InventoryItem(Box::new(InventoryItem::new(
                    data.item.0.clone(),
                    item_display.string_tag.0.clone(),
                ))),
            );
        }

        let icon_tag: SInvestmentIcons = package_manager().read_tag_struct(
            package_manager().get_all_by_reference(SInvestmentIcons::ID.unwrap())[0].0,
        )?;

        Ok(Self {
            strings: Arc::new(StringManager {
                string_cache: RwLock::new(FxHashMap::default()),
                indexed_strings: package_manager().read_tag_struct(
                    package_manager().get_all_by_reference(SIndexedLocalizedStrings::ID.unwrap())
                        [0]
                    .0,
                )?,
            }),
            activities: Arc::new(ActivityManager { activities }),
            items: Arc::new(ItemManager { items: item_map }),

            icons: Arc::new(IconManager {
                icon_tag,
                icon_cache: RwLock::new(FxHashMap::default()),
            }),
        })
    }

    pub fn activities(&self) -> Arc<ActivityManager> {
        self.activities.clone()
    }

    pub fn strings(&self) -> Arc<StringManager> {
        self.strings.clone()
    }

    pub fn items(&self) -> Arc<ItemManager> {
        self.items.clone()
    }

    pub fn icons(&self) -> Arc<IconManager> {
        self.icons.clone()
    }

    // TODO: search by partial hash (convert to string + contains?)
    /// Get anything from a hash. Scans all tables for any containing this hash and returns a vec of results.
    #[tracing::instrument(skip(self))]
    pub fn search_by_hash(&self, hash: u32) -> Vec<InvestmentData> {
        let mut results = Vec::new();

        if let Some(act) = self.activities.get_by_hash(hash) {
            results.push(act);
        }

        if let Some(item) = self.items.get_by_hash(hash) {
            results.push(item);
        }

        results
    }

    /// Get anything from a name. Scans all tables for any containing "name" and returns a vec of results.
    #[tracing::instrument(skip(self))]
    pub fn search_by_name(
        &self,
        search_channel: Arc<crossbeam::channel::Sender<InvestmentData>>,
        language: Language,
        name: String,
    ) {
        let act = self.activities.clone();
        let act_channel = search_channel.clone();
        let name1 = name.to_lowercase().clone();
        std::thread::spawn(move || act.search_by_name(act_channel, language, name1));
        let item_channel = search_channel.clone();
        let items = self.items.clone();
        let name1 = name.to_lowercase().clone();
        std::thread::spawn(move || items.search_by_name(item_channel, language, name1));
    }
}

impl StringManager {
    pub fn get_indexed_string(&self, language: Language, index: u32, hash: u32) -> Option<String> {
        if let Some(cached) = self.string_cache.read().get(&(index, hash)) {
            return Some(cached.to_owned());
        }
        let strings_data = self.indexed_strings.localized_strings.get(index as usize)?;

        let Ok(loc) = LocalizedStrings::load(strings_data.localized_tag) else {
            return None;
        };
        let string = loc.get(&language, hash)?;
        self.string_cache
            .write()
            .insert((index, hash), string.to_owned());
        Some(string.to_owned())
    }
}

impl ActivityManager {
    /// Returns a Vec containing the activities with `name` in their name.
    #[tracing::instrument(skip(self))]
    pub fn search_by_name(
        &self,
        channel: Arc<crossbeam::channel::Sender<InvestmentData>>,
        language: Language,
        name: String,
    ) {
        if self
            .activities
            .values()
            .par_iter()
            .try_for_each(move |act| -> anyhow::Result<()> {
                if let InvestmentData::Activity(data, display) = act {
                    let act_name = display.display_properties.name.get(language);
                    if act_name
                        .clone()
                        .is_some_and(|n| n.to_lowercase().contains(&name))
                        || act_name.is_none() && name.is_empty()
                    {
                        channel
                            .try_send(InvestmentData::Activity(data.clone(), display.clone()))?;
                    }
                }
                Ok(())
            })
            .is_err()
        {
            // Silently error, for now?
        };
    }

    /// Returns an activity's data by it's hash.
    #[tracing::instrument(skip(self))]
    pub fn get_by_hash(&self, hash: u32) -> Option<InvestmentData> {
        self.activities.lookup_hash(&hash).cloned()
    }

    /// Returns an activity's data by it's index in the table.
    #[tracing::instrument(skip(self))]
    pub fn get_activity_by_index(&self, index: u16) -> Option<InvestmentData> {
        self.activities.get(index as usize).cloned()
    }
}

impl ItemManager {
    /// Returns a Vec containing the items with `name` in their name.
    #[tracing::instrument(skip(self))]
    pub fn search_by_name(
        &self,
        channel: Arc<crossbeam::channel::Sender<InvestmentData>>,
        language: Language,
        name: String,
    ) {
        if self
            .items
            .values()
            .par_iter()
            .try_for_each(move |item| -> anyhow::Result<()> {
                if let InvestmentData::InventoryItem(i) = item {
                    let act_name = i.display.name.get(language);
                    if act_name
                        .clone()
                        .is_some_and(|n| n.to_lowercase().contains(&name))
                        || act_name.is_none() && name.is_empty()
                    {
                        channel.try_send(InvestmentData::InventoryItem(i.clone()))?;
                    }
                }
                Ok(())
            })
            .is_err()
        {
            // Silently error, for now?
        };
    }

    /// Returns an item's data by it's hash.
    #[tracing::instrument(skip(self))]
    pub fn get_by_hash(&self, hash: u32) -> Option<InvestmentData> {
        self.items.lookup_hash(&hash).cloned()
    }

    /// Returns an item's data by it's index in the table.
    #[tracing::instrument(skip(self))]
    pub fn get_activity_by_index(&self, index: u16) -> Option<InvestmentData> {
        self.items.get(index as usize).cloned()
    }
}

impl IconManager {
    pub fn get(&self, index: usize) -> Option<SInvestmentIcon> {
        if let Some(icon_cached) = self.icon_cache.read().get(&index) {
            return Some(icon_cached.clone());
        }
        if let Some(icon) = self.icon_tag.icons.get(index).map(|x| {
            package_manager()
                .read_tag_struct::<SInvestmentIcon>(x.icon)
                .unwrap()
        }) {
            self.icon_cache.write().insert(index, icon.clone());
            return Some(icon);
        };
        None
    }
}
