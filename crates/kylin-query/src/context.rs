use kylin_metadata::DataModel;
use kylin_metadata::dataflow::{Dataflow, LayoutEntity};
use kylin_catalog::LayoutCandidate;

/// Olap context - represents a sub-query that can be answered by a single model
#[derive(Debug, Clone)]
pub struct OlapQueryContext {
    pub id: usize,
    pub project: String,
    pub model: Option<DataModel>,
    pub dataflow: Option<Dataflow>,
    pub matched_layout: Option<LayoutCandidate>,
    pub referenced_tables: Vec<String>,
    pub referenced_columns: Vec<String>,
    pub is_matched: bool,
}

impl OlapQueryContext {
    pub fn new(id: usize, project: &str) -> Self {
        Self {
            id,
            project: project.to_string(),
            model: None,
            dataflow: None,
            matched_layout: None,
            referenced_tables: Vec::new(),
            referenced_columns: Vec::new(),
            is_matched: false,
        }
    }

    /// Set the model and dataflow for this context
    pub fn set_model(&mut self, model: DataModel, dataflow: Dataflow) {
        self.model = Some(model);
        self.dataflow = Some(dataflow);
    }

    /// Set the matched layout
    pub fn set_matched_layout(&mut self, layout: LayoutCandidate) {
        self.matched_layout = Some(layout);
        self.is_matched = true;
    }

    /// Add a referenced table
    pub fn add_table(&mut self, table: &str) {
        if !self.referenced_tables.contains(&table.to_string()) {
            self.referenced_tables.push(table.to_string());
        }
    }

    /// Add a referenced column
    pub fn add_column(&mut self, column: &str) {
        if !self.referenced_columns.contains(&column.to_string()) {
            self.referenced_columns.push(column.to_string());
        }
    }

    /// Get the model name
    pub fn model_name(&self) -> Option<&str> {
        self.model.as_ref().map(|m| m.name.as_str())
    }

    /// Get the layout ID if matched
    pub fn layout_id(&self) -> Option<i64> {
        self.matched_layout.as_ref().map(|l| l.layout.id)
    }
}

/// Query analyzer - extracts information from SQL queries
#[derive(Debug)]
pub struct QueryAnalyzer {
    project: String,
}

impl QueryAnalyzer {
    pub fn new(project: &str) -> Self {
        Self {
            project: project.to_string(),
        }
    }

    /// Analyze a SQL query and extract referenced tables and columns
    pub fn analyze(&self, sql: &str) -> Vec<OlapQueryContext> {
        // Simple analysis - create a single context for the query
        // In a real implementation, this would parse the SQL and extract
        // table and column references
        let mut context = OlapQueryContext::new(0, &self.project);

        // Extract table names from SQL (simplified)
        // A real implementation would use the SQL parser
        let tables = extract_tables_from_sql(sql);
        for table in tables {
            context.add_table(&table);
        }

        // Extract column names from SQL (simplified)
        let columns = extract_columns_from_sql(sql);
        for column in columns {
            context.add_column(&column);
        }

        vec![context]
    }
}

/// Extract table names from SQL (simplified implementation)
fn extract_tables_from_sql(sql: &str) -> Vec<String> {
    let sql_upper = sql.to_uppercase();
    let mut tables = Vec::new();

    // Simple extraction - look for FROM and JOIN clauses
    let parts: Vec<&str> = sql_upper.split_whitespace().collect();
    let mut i = 0;
    while i < parts.len() {
        if (parts[i] == "FROM" || parts[i] == "JOIN") && i + 1 < parts.len() {
            let table = parts[i + 1].trim_end_matches(',');
            if !table.is_empty() && !table.starts_with('(') {
                tables.push(table.to_lowercase());
            }
        }
        i += 1;
    }

    tables
}

/// Extract column names from SQL (simplified implementation)
fn extract_columns_from_sql(sql: &str) -> Vec<String> {
    let sql_upper = sql.to_uppercase();
    let mut columns = Vec::new();

    // Simple extraction - look for SELECT clause
    if let Some(select_pos) = sql_upper.find("SELECT") {
        let after_select = &sql[select_pos + 6..];
        if let Some(from_pos) = after_select.to_uppercase().find("FROM") {
            let select_clause = &after_select[..from_pos];
            for col in select_clause.split(',') {
                let col = col.trim();
                if col != "*" {
                    // Extract column name (handle aliases)
                    let parts: Vec<&str> = col.split_whitespace().collect();
                    if !parts.is_empty() {
                        let col_name = parts[0].split('.').next_back().unwrap_or(parts[0]);
                        columns.push(col_name.to_lowercase());
                    }
                }
            }
        }
    }

    columns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_olap_context() {
        let mut ctx = OlapQueryContext::new(0, "test_project");
        assert_eq!(ctx.project, "test_project");
        assert!(!ctx.is_matched);

        ctx.add_table("sales");
        ctx.add_column("id");
        ctx.add_column("amount");

        assert_eq!(ctx.referenced_tables.len(), 1);
        assert_eq!(ctx.referenced_columns.len(), 2);
    }

    #[test]
    fn test_query_analyzer() {
        let analyzer = QueryAnalyzer::new("test_project");
        let contexts = analyzer.analyze("SELECT id, amount FROM sales WHERE id > 100");

        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].referenced_tables, vec!["sales"]);
        assert!(contexts[0].referenced_columns.contains(&"id".to_string()));
        assert!(contexts[0].referenced_columns.contains(&"amount".to_string()));
    }

    #[test]
    fn test_extract_tables() {
        let tables = extract_tables_from_sql("SELECT * FROM sales JOIN products ON sales.product_id = products.id");
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"sales".to_string()));
        assert!(tables.contains(&"products".to_string()));
    }

    #[test]
    fn test_extract_columns() {
        let columns = extract_columns_from_sql("SELECT id, name, amount FROM sales");
        assert_eq!(columns.len(), 3);
        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"amount".to_string()));
    }
}
