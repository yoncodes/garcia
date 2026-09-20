use std::{collections::HashMap, fs, hash::Hash, path::Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, de::DeserializeOwned};

#[derive(Debug, Deserialize)]
pub struct TableFile<T> {
    #[serde(rename = "list")]
    pub rows: Vec<T>,
}

impl<T: DeserializeOwned> TableFile<T> {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).with_context(|| format!("could not read {}", path.display()))?;
        serde_json::from_slice(&bytes)
            .with_context(|| format!("could not parse {}", path.display()))
    }
}

#[derive(Debug)]
pub struct Table<K, T> {
    pub rows: Vec<T>,
    index: HashMap<K, usize>,
}

impl<K: Eq + Hash, T> Table<K, T> {
    pub fn new(file: TableFile<T>, table_name: &str, key: impl Fn(&T) -> K) -> Result<Self> {
        let mut index = HashMap::with_capacity(file.rows.len());
        for (position, row) in file.rows.iter().enumerate() {
            let key = key(row);
            if index.insert(key, position).is_some() {
                bail!("duplicate key in {table_name}");
            }
        }
        Ok(Self {
            rows: file.rows,
            index,
        })
    }
}

impl<T> Table<i32, T> {
    pub fn get(&self, key: i32) -> Option<&T> {
        self.index.get(&key).map(|&position| &self.rows[position])
    }
}

impl<T> Table<String, T> {
    pub fn get(&self, key: &str) -> Option<&T> {
        self.index.get(key).map(|&position| &self.rows[position])
    }
}

#[derive(Debug)]
pub struct GroupedTable<K, T> {
    pub rows: Vec<T>,
    groups: HashMap<K, Vec<usize>>,
}

impl<K: Eq + Hash, T> GroupedTable<K, T> {
    pub fn new(file: TableFile<T>, key: impl Fn(&T) -> K) -> Self {
        let mut groups: HashMap<K, Vec<usize>> = HashMap::new();
        for (position, row) in file.rows.iter().enumerate() {
            groups.entry(key(row)).or_default().push(position);
        }
        Self {
            rows: file.rows,
            groups,
        }
    }
}

impl<T> GroupedTable<i32, T> {
    pub fn get(&self, key: i32) -> Option<impl Iterator<Item = &T>> {
        self.groups
            .get(&key)
            .map(|positions| positions.iter().map(|&position| &self.rows[position]))
    }
}

pub(crate) fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("could not read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("could not parse {}", path.display()))
}
