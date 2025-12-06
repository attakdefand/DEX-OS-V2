//! Indexer service with filtering engine
//!
//! This module implements the Priority 3 feature from DEX-OS-V2.csv:
//! - Core Trading,Indexer,Indexer,Filtering Engine,Selective Data Capture,Medium
//! - Infrastructure,Indexer,Indexer,Materialized Views,Data Aggregation,Medium

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Indexer service with filtering capabilities and materialized views
#[derive(Debug, Clone)]
pub struct IndexerService {
    /// Data filters
    filters: HashMap<String, DataFilter>,
    /// Indexed data entries
    entries: Vec<IndexedData>,
    /// Index for quick lookup by filter
    filter_index: HashMap<String, Vec<usize>>,
    /// Materialized views for aggregated data
    materialized_views: HashMap<String, MaterializedView>,
    /// Maximum number of entries to store
    max_entries: usize,
}

/// Data filter for selective indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFilter {
    /// Filter identifier
    pub id: String,
    /// Filter name
    pub name: String,
    /// Filter criteria
    pub criteria: FilterCriteria,
    /// Whether the filter is active
    pub active: bool,
    /// Creation timestamp
    pub created_at: u64,
}

/// Criteria for filtering data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCriteria {
    /// Data types to include
    pub data_types: Vec<String>,
    /// Tags to match (all must match)
    pub tags: Vec<String>,
    /// Tags to exclude (any match excludes)
    pub exclude_tags: Vec<String>,
    /// Minimum priority level
    pub min_priority: Option<u32>,
    /// Custom filter function (represented as string for serialization)
    pub custom_filter: Option<String>,
}

/// Indexed data entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedData {
    /// Entry identifier
    pub id: String,
    /// Data type
    pub data_type: String,
    /// Data content (serialized)
    pub content: String,
    /// Tags associated with the data
    pub tags: Vec<String>,
    /// Priority level
    pub priority: u32,
    /// Timestamp when indexed
    pub indexed_at: u64,
    /// Which filters matched this entry
    pub matched_filters: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Materialized view for aggregated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedView {
    /// View identifier
    pub id: String,
    /// View name
    pub name: String,
    /// Filter ID this view is based on
    pub filter_id: String,
    /// Aggregated data
    pub aggregated_data: String,
    /// Last update timestamp
    pub last_updated: u64,
    /// View configuration
    pub config: MaterializedViewConfig,
}

/// Configuration for materialized views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedViewConfig {
    /// Aggregation function to apply
    pub aggregation_function: String,
    /// Group by fields
    pub group_by: Vec<String>,
    /// Refresh interval in seconds (0 for manual refresh only)
    pub refresh_interval: u64,
    /// Whether to automatically refresh when new data arrives
    pub auto_refresh: bool,
}

impl IndexerService {
    /// Create a new indexer service
    pub fn new(max_entries: usize) -> Self {
        Self {
            filters: HashMap::new(),
            entries: Vec::new(),
            filter_index: HashMap::new(),
            materialized_views: HashMap::new(),
            max_entries,
        }
    }

    /// Add a new data filter
    pub fn add_filter(&mut self, filter: DataFilter) -> Result<(), IndexerError> {
        if self.filters.contains_key(&filter.id) {
            return Err(IndexerError::FilterAlreadyExists);
        }

        let filter_id = filter.id.clone();
        // Initialize filter index
        self.filter_index.insert(filter_id.clone(), Vec::new());

        self.filters.insert(filter_id, filter);
        Ok(())
    }

    /// Update an existing filter
    pub fn update_filter(&mut self, filter: DataFilter) -> Result<(), IndexerError> {
        if !self.filters.contains_key(&filter.id) {
            return Err(IndexerError::FilterNotFound);
        }

        let filter_id = filter.id.clone();
        self.filters.insert(filter_id, filter);
        Ok(())
    }

    /// Remove a filter
    pub fn remove_filter(&mut self, filter_id: &str) -> Result<(), IndexerError> {
        if !self.filters.contains_key(filter_id) {
            return Err(IndexerError::FilterNotFound);
        }

        self.filters.remove(filter_id);
        self.filter_index.remove(filter_id);

        // Remove references to this filter from entries
        for entry in &mut self.entries {
            entry.matched_filters.retain(|id| id != filter_id);
        }

        // Remove any materialized views based on this filter
        self.materialized_views
            .retain(|_, view| view.filter_id != filter_id);

        Ok(())
    }

    /// Get a filter by ID
    pub fn get_filter(&self, filter_id: &str) -> Option<&DataFilter> {
        self.filters.get(filter_id)
    }

    /// Get all filters
    pub fn get_all_filters(&self) -> Vec<&DataFilter> {
        self.filters.values().collect()
    }

    /// Index new data entry
    pub fn index_data(
        &mut self,
        data_type: String,
        content: String,
        tags: Vec<String>,
        priority: u32,
        metadata: HashMap<String, String>,
    ) -> Result<String, IndexerError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry_id = format!("entry_{}", now);

        // Determine which filters match this data
        let matched_filters = self.find_matching_filters(&data_type, &tags, priority);

        let entry = IndexedData {
            id: entry_id.clone(),
            data_type,
            content,
            tags,
            priority,
            indexed_at: now,
            matched_filters: matched_filters.clone(),
            metadata,
        };

        // Add entry to storage
        self.entries.push(entry);

        // Update filter indexes
        for filter_id in matched_filters {
            if let Some(index) = self.filter_index.get_mut(&filter_id) {
                index.push(self.entries.len() - 1);
            }
        }

        // Refresh materialized views that auto-refresh
        self.refresh_auto_materialized_views()?;

        // Trim entries if we exceed max_entries
        if self.entries.len() > self.max_entries {
            self.trim_entries();
        }

        Ok(entry_id)
    }

    /// Find entries matching a filter
    pub fn find_entries_by_filter(
        &self,
        filter_id: &str,
    ) -> Result<Vec<&IndexedData>, IndexerError> {
        if !self.filters.contains_key(filter_id) {
            return Err(IndexerError::FilterNotFound);
        }

        if let Some(indices) = self.filter_index.get(filter_id) {
            let entries: Vec<&IndexedData> = indices
                .iter()
                .filter_map(|&index| self.entries.get(index))
                .collect();
            Ok(entries)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find entries matching criteria directly
    pub fn find_entries_by_criteria(
        &self,
        data_types: &[String],
        tags: &[String],
        min_priority: Option<u32>,
    ) -> Vec<&IndexedData> {
        self.entries
            .iter()
            .filter(|entry| {
                // Check data type
                if !data_types.is_empty() && !data_types.contains(&entry.data_type) {
                    return false;
                }

                // Check tags (all must match)
                for tag in tags {
                    if !entry.tags.contains(tag) {
                        return false;
                    }
                }

                // Check priority
                if let Some(min) = min_priority {
                    if entry.priority < min {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get recent entries
    pub fn get_recent_entries(&self, count: usize) -> Vec<&IndexedData> {
        let skip = if self.entries.len() > count {
            self.entries.len() - count
        } else {
            0
        };
        self.entries.iter().skip(skip).collect()
    }

    /// Create a materialized view
    pub fn create_materialized_view(
        &mut self,
        id: String,
        name: String,
        filter_id: String,
        config: MaterializedViewConfig,
    ) -> Result<(), IndexerError> {
        if !self.filters.contains_key(&filter_id) {
            return Err(IndexerError::FilterNotFound);
        }

        if self.materialized_views.contains_key(&id) {
            return Err(IndexerError::ViewAlreadyExists);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create initial aggregated data
        let aggregated_data = self.aggregate_data(&filter_id, &config)?;

        let view = MaterializedView {
            id: id.clone(),
            name,
            filter_id,
            aggregated_data,
            last_updated: now,
            config,
        };

        self.materialized_views.insert(id, view);
        Ok(())
    }

    /// Refresh a materialized view
    pub fn refresh_materialized_view(&mut self, view_id: &str) -> Result<(), IndexerError> {
        if !self.materialized_views.contains_key(view_id) {
            return Err(IndexerError::ViewNotFound);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let view = self
            .materialized_views
            .get(&view_id.to_string())
            .unwrap()
            .clone();
        let aggregated_data = self.aggregate_data(&view.filter_id, &view.config)?;

        // Update the view
        let view = self.materialized_views.get_mut(view_id).unwrap();
        view.aggregated_data = aggregated_data;
        view.last_updated = now;

        Ok(())
    }

    /// Get a materialized view
    pub fn get_materialized_view(&self, view_id: &str) -> Option<&MaterializedView> {
        self.materialized_views.get(view_id)
    }

    /// Get all materialized views
    pub fn get_all_materialized_views(&self) -> Vec<&MaterializedView> {
        self.materialized_views.values().collect()
    }

    /// Remove a materialized view
    pub fn remove_materialized_view(&mut self, view_id: &str) -> Result<(), IndexerError> {
        if !self.materialized_views.contains_key(view_id) {
            return Err(IndexerError::ViewNotFound);
        }

        self.materialized_views.remove(view_id);
        Ok(())
    }

    /// Find which filters match given data characteristics
    fn find_matching_filters(
        &self,
        data_type: &str,
        tags: &[String],
        priority: u32,
    ) -> Vec<String> {
        let mut matching_filters = Vec::new();

        for (filter_id, filter) in &self.filters {
            if !filter.active {
                continue;
            }

            let criteria = &filter.criteria;

            // Check data types
            if !criteria.data_types.is_empty()
                && !criteria.data_types.contains(&data_type.to_string())
            {
                continue;
            }

            // Check exclude tags (any match excludes)
            let mut excluded = false;
            for exclude_tag in &criteria.exclude_tags {
                if tags.contains(exclude_tag) {
                    excluded = true;
                    break;
                }
            }
            if excluded {
                continue;
            }

            // Check required tags (all must match)
            let mut all_tags_match = true;
            for required_tag in &criteria.tags {
                if !tags.contains(required_tag) {
                    all_tags_match = false;
                    break;
                }
            }
            if !all_tags_match {
                continue;
            }

            // Check priority
            if let Some(min_priority) = criteria.min_priority {
                if priority < min_priority {
                    continue;
                }
            }

            // If we get here, the filter matches
            matching_filters.push(filter_id.clone());
        }

        matching_filters
    }

    /// Aggregate data for a materialized view
    fn aggregate_data(
        &self,
        filter_id: &str,
        config: &MaterializedViewConfig,
    ) -> Result<String, IndexerError> {
        let entries = self.find_entries_by_filter(filter_id)?;

        // For now, we'll implement a simple count aggregation as an example
        // In a real implementation, this would be more sophisticated
        let count = entries.len();

        // Group by implementation would go here
        // For now, we'll just return a simple JSON representation
        let result = format!(
            "{{\"count\": {}, \"aggregation_function\": \"{}\"}}",
            count, config.aggregation_function
        );

        Ok(result)
    }

    /// Refresh materialized views that are set to auto-refresh
    fn refresh_auto_materialized_views(&mut self) -> Result<(), IndexerError> {
        let view_ids: Vec<String> = self
            .materialized_views
            .iter()
            .filter(|(_, view)| view.config.auto_refresh)
            .map(|(id, _)| id.clone())
            .collect();

        for view_id in view_ids {
            self.refresh_materialized_view(&view_id)?;
        }

        Ok(())
    }

    /// Trim old entries to maintain max_entries limit
    fn trim_entries(&mut self) {
        let excess = self.entries.len() - self.max_entries;
        if excess > 0 {
            // Remove oldest entries
            self.entries.drain(0..excess);

            // Rebuild filter indexes since entry indices have changed
            self.rebuild_filter_indexes();
        }
    }

    /// Rebuild filter indexes after entries have been removed
    fn rebuild_filter_indexes(&mut self) {
        // Clear existing indexes
        for index in self.filter_index.values_mut() {
            index.clear();
        }

        // Rebuild indexes
        for (i, entry) in self.entries.iter().enumerate() {
            for filter_id in &entry.matched_filters {
                if let Some(index) = self.filter_index.get_mut(filter_id) {
                    index.push(i);
                }
            }
        }
    }
}

impl Default for IndexerService {
    fn default() -> Self {
        Self::new(10000) // Default to storing 10,000 entries
    }
}

/// Errors that can occur during indexer operations
#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Filter already exists")]
    FilterAlreadyExists,
    #[error("Filter not found")]
    FilterNotFound,
    #[error("View already exists")]
    ViewAlreadyExists,
    #[error("View not found")]
    ViewNotFound,
}
