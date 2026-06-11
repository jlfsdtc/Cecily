use kylin_metadata::DataModel;
use kylin_metadata::dataflow::{Dataflow, LayoutEntity};

/// Layout candidate for query answering
#[derive(Debug, Clone)]
pub struct LayoutCandidate {
    pub layout: LayoutEntity,
    pub is_imperfect_match: bool,
    pub matched_dimensions: Vec<String>,
    pub matched_measures: Vec<String>,
    pub cost: f64,
}

/// Layout chooser - selects the best layout for a query
pub struct LayoutChooser;

impl LayoutChooser {
    /// Choose the best layout for a given query
    pub fn choose_best_layout(
        model: &DataModel,
        dataflow: &Dataflow,
        required_columns: &[String],
    ) -> Option<LayoutCandidate> {
        if dataflow.layouts.is_empty() {
            return None;
        }

        let mut candidates: Vec<LayoutCandidate> = dataflow
            .layouts
            .iter()
            .filter_map(|layout| Self::evaluate_layout(model, layout, required_columns))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by: exact match first, then by cost
        candidates.sort_by(|a, b| {
            a.is_imperfect_match
                .cmp(&b.is_imperfect_match)
                .then(a.cost.partial_cmp(&b.cost).unwrap_or(std::cmp::Ordering::Equal))
        });

        candidates.into_iter().next()
    }

    /// Evaluate a layout against required columns
    fn evaluate_layout(
        model: &DataModel,
        layout: &LayoutEntity,
        required_columns: &[String],
    ) -> Option<LayoutCandidate> {
        // Get dimension names from layout
        let layout_dims: Vec<String> = layout
            .dimensions
            .iter()
            .filter_map(|dim_id| {
                model
                    .all_columns
                    .iter()
                    .find(|c| c.uuid == *dim_id)
                    .map(|c| c.name.clone())
            })
            .collect();

        // Get measure names from layout
        let layout_measures: Vec<String> = layout
            .measures
            .iter()
            .filter_map(|measure_id| {
                model
                    .all_measures
                    .iter()
                    .find(|m| m.uuid == *measure_id)
                    .map(|m| m.name.clone())
            })
            .collect();

        // Check if layout contains all required columns
        let matched_dims: Vec<String> = required_columns
            .iter()
            .filter(|col| layout_dims.contains(col))
            .cloned()
            .collect();

        let matched_measures: Vec<String> = required_columns
            .iter()
            .filter(|col| layout_measures.contains(col))
            .cloned()
            .collect();

        let total_required = required_columns.len();
        let total_matched = matched_dims.len() + matched_measures.len();

        if total_matched == 0 {
            return None;
        }

        let is_imperfect_match = total_matched < total_required;

        // Calculate cost based on:
        // 1. Number of matched columns (more is better)
        // 2. Layout type (table index is cheaper)
        // 3. Storage size (smaller is better)
        let match_ratio = total_matched as f64 / total_required as f64;
        let type_bonus = if layout.is_table_index { 0.1 } else { 0.0 };
        let size_factor = if layout.storage_size > 0 {
            layout.storage_size as f64 / 1_000_000.0 // Normalize to MB
        } else {
            1.0
        };

        let cost = (1.0 - match_ratio) + type_bonus + size_factor * 0.001;

        Some(LayoutCandidate {
            layout: layout.clone(),
            is_imperfect_match,
            matched_dimensions: matched_dims,
            matched_measures: matched_measures,
            cost,
        })
    }

    /// Find all layouts that can answer the query
    pub fn find_matching_layouts(
        model: &DataModel,
        dataflow: &Dataflow,
        required_columns: &[String],
    ) -> Vec<LayoutCandidate> {
        dataflow
            .layouts
            .iter()
            .filter_map(|layout| Self::evaluate_layout(model, layout, required_columns))
            .collect()
    }

    /// Find the best table index layout
    pub fn find_best_table_index(
        model: &DataModel,
        dataflow: &Dataflow,
    ) -> Option<LayoutCandidate> {
        let table_indexes: Vec<LayoutCandidate> = dataflow
            .layouts
            .iter()
            .filter(|l| l.is_table_index)
            .map(|layout| {
                let dims: Vec<String> = layout
                    .dimensions
                    .iter()
                    .filter_map(|dim_id| {
                        model
                            .all_columns
                            .iter()
                            .find(|c| c.uuid == *dim_id)
                            .map(|c| c.name.clone())
                    })
                    .collect();

                LayoutCandidate {
                    layout: layout.clone(),
                    is_imperfect_match: false,
                    matched_dimensions: dims,
                    matched_measures: vec![],
                    cost: 0.0,
                }
            })
            .collect();

        table_indexes.into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_common::types::{KylinDataType, ModelType, PersistentEntity};
    use kylin_metadata::dataflow::DataflowStatus;
    use kylin_metadata::model::ColumnDesc;

    fn create_test_model() -> DataModel {
        DataModel {
            entity: PersistentEntity::new(),
            name: "test_model".to_string(),
            root_fact_table: "DEFAULT.SALES".to_string(),
            model_type: ModelType::Batch,
            join_tables: vec![],
            all_columns: vec![
                ColumnDesc {
                    uuid: "col1".to_string(),
                    name: "id".to_string(),
                    data_type: KylinDataType::BigInt,
                    is_dimension: true,
                    is_computed: false,
                    table_name: "SALES".to_string(),
                    comment: None,
                },
                ColumnDesc {
                    uuid: "col2".to_string(),
                    name: "category".to_string(),
                    data_type: KylinDataType::String,
                    is_dimension: true,
                    is_computed: false,
                    table_name: "SALES".to_string(),
                    comment: None,
                },
                ColumnDesc {
                    uuid: "col3".to_string(),
                    name: "amount".to_string(),
                    data_type: KylinDataType::Double,
                    is_dimension: false,
                    is_computed: false,
                    table_name: "SALES".to_string(),
                    comment: None,
                },
            ],
            all_measures: vec![],
            filter_condition: None,
            partition_desc: None,
            computed_columns: vec![],
        }
    }

    fn create_test_dataflow(model: &DataModel) -> Dataflow {
        Dataflow {
            entity: PersistentEntity::new(),
            project: "test_project".to_string(),
            model_uuid: model.entity.uuid.clone(),
            model_name: model.name.clone(),
            status: DataflowStatus::Active,
            segments: vec![],
            layouts: vec![
                LayoutEntity {
                    id: 1,
                    dimensions: vec!["col1".to_string(), "col2".to_string()],
                    measures: vec![],
                    shard_by_columns: vec![],
                    is_table_index: true,
                    storage_size: 1000000,
                    row_count: 10000,
                },
                LayoutEntity {
                    id: 2,
                    dimensions: vec!["col1".to_string()],
                    measures: vec![],
                    shard_by_columns: vec![],
                    is_table_index: false,
                    storage_size: 500000,
                    row_count: 10000,
                },
            ],
        }
    }

    #[test]
    fn test_choose_best_layout() {
        let model = create_test_model();
        let dataflow = create_test_dataflow(&model);

        // Query needs col1 and col2
        let required = vec!["id".to_string(), "category".to_string()];
        let candidate = LayoutChooser::choose_best_layout(&model, &dataflow, &required);

        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert_eq!(candidate.layout.id, 1); // Table index with both columns
        assert!(!candidate.is_imperfect_match);
    }

    #[test]
    fn test_choose_layout_imperfect_match() {
        let model = create_test_model();
        let dataflow = create_test_dataflow(&model);

        // Query needs col1, col2, and col3
        let required = vec!["id".to_string(), "category".to_string(), "amount".to_string()];
        let candidate = LayoutChooser::choose_best_layout(&model, &dataflow, &required);

        assert!(candidate.is_some());
        let candidate = candidate.unwrap();
        assert!(candidate.is_imperfect_match);
    }

    #[test]
    fn test_find_matching_layouts() {
        let model = create_test_model();
        let dataflow = create_test_dataflow(&model);

        let required = vec!["id".to_string()];
        let candidates = LayoutChooser::find_matching_layouts(&model, &dataflow, &required);

        assert_eq!(candidates.len(), 2); // Both layouts match
    }
}
