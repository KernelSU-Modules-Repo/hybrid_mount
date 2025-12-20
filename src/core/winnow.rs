use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WinnowingTable {
    #[serde(flatten)]
    pub rules: HashMap<String, String>, 
}

impl Default for WinnowingTable {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
}

impl WinnowingTable {
    pub fn get_preferred_module(&self, file_path: &Path) -> Option<String> {
        let path_str = file_path.to_string_lossy().to_string();
        self.rules.get(&path_str).cloned()
    }

    pub fn set_rule(&mut self, file_path: &str, module_id: &str) {
        self.rules.insert(file_path.to_string(), module_id.to_string());
    }
    
    pub fn remove_rule(&mut self, file_path: &str) {
        self.rules.remove(file_path);
    }
}

#[derive(Debug)]
pub struct ChaffConflict {
    pub path: PathBuf,
    pub contenders: Vec<String>,
    pub selected: String,
    pub is_forced: bool,
}

pub fn sift_conflicts(
    conflicts: Vec<crate::core::planner::ConflictDetail>,
    table: &WinnowingTable
) -> Vec<ChaffConflict> {
    conflicts.into_iter().map(|c| {
        let path_str = format!("/system/{}", c.relative_path); 
        let forced_module = table.get_preferred_module(Path::new(&path_str));
        
        let selected = if let Some(forced) = &forced_module {
            if c.contending_modules.contains(forced) {
                forced.clone()
            } else {
                c.contending_modules.last().unwrap_or(&"unknown".to_string()).clone()
            }
        } else {
            c.contending_modules.last().unwrap_or(&"unknown".to_string()).clone()
        };

        ChaffConflict {
            path: PathBuf::from(path_str),
            contenders: c.contending_modules,
            selected,
            is_forced: forced_module.is_some(),
        }
    }).collect()
}