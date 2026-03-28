// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Cursor,
    path::Path,
};

use anyhow::Result;
use java_properties::PropertiesIter;
use serde::Serialize;

use crate::{
    defs::{DISABLE_FILE_NAME, REMOVE_FILE_NAME, SKIP_MOUNT_FILE_NAME},
    utils::validate_module_id,
};

#[derive(Debug, Serialize)]
pub struct ModuleInfo {
    pub id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    disabled: bool,
    skip: bool,
}

fn normalize_version(version: &str) -> String {
    let mut chars = version.chars();
    match chars.next() {
        Some(prefix @ ('v' | 'V')) if chars.next().is_some_and(|ch| ch.is_ascii_digit()) => {
            let stripped = version.strip_prefix(prefix).unwrap_or(version);
            stripped.to_string()
        }
        _ => version.to_string(),
    }
}

fn read_prop<P>(path: P) -> Result<HashMap<String, String>>
where
    P: AsRef<Path>,
{
    let buffer = fs::read_to_string(path)?;
    let mut map = HashMap::new();
    PropertiesIter::new_with_encoding(Cursor::new(buffer), encoding_rs::UTF_8).read_into(
        |k, v| {
            map.insert(k, v);
        },
    )?;

    Ok(map)
}

/// Scans for modules that will be actually mounted by `magic_mount`.
/// Filters out modules that:
/// 1. Do not have a `system` directory.
/// 2. Are disabled or removed.
/// 3. Have the `skip_mount` flag.
pub fn scan_modules<P>(module_dir: P, extra: &[String]) -> Vec<ModuleInfo>
where
    P: AsRef<Path>,
{
    let mut modules = Vec::new();

    if let Ok(entries) = module_dir.as_ref().read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            if !path.join("module.prop").exists() {
                continue;
            }

            let mut modified = false;
            let mut partitions = HashSet::new();
            partitions.insert("system".to_string());
            partitions.extend(extra.iter().cloned());

            for p in &partitions {
                if entry.path().join(p).is_dir() {
                    modified = true;
                    break;
                }
            }

            if !modified {
                continue;
            }

            let disabled =
                path.join(DISABLE_FILE_NAME).exists() || path.join(REMOVE_FILE_NAME).exists();
            let skip = path.join(SKIP_MOUNT_FILE_NAME).exists();
            if disabled || skip {
                continue;
            }

            let prop_path = path.join("module.prop");

            let Ok(prop) = read_prop(prop_path) else {
                continue;
            };
            let Some(id) = prop.get("id") else {
                log::warn!("{} missing module id", path.display());
                continue;
            };
            let Some(name) = prop.get("name") else {
                log::warn!("{} missing module name", path.display());
                continue;
            };
            let Some(version) = prop.get("version") else {
                log::warn!("{} missing module version", path.display());
                continue;
            };
            let Some(author) = prop.get("author") else {
                log::warn!("{} missing module author", path.display());
                continue;
            };
            let Some(description) = prop.get("description") else {
                log::warn!("{} missing module description", path.display());
                continue;
            };

            if validate_module_id(id).is_ok() {
                modules.push(ModuleInfo {
                    id: id.clone(),
                    name: name.clone(),
                    version: normalize_version(version),
                    author: author.clone(),
                    description: description.clone(),
                    disabled,
                    skip,
                });
            }
        }
    }
    modules.sort_by(|a, b| a.id.cmp(&b.id));

    modules
}
