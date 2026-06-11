use arrow::record_batch::RecordBatch;
use kylin_common::{KylinError, Result};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::Path;

/// Parquet writer utility
pub struct ParquetWriter {
    compression: parquet::basic::Compression,
    batch_size: usize,
}

impl ParquetWriter {
    /// Create a new Parquet writer
    pub fn new() -> Self {
        Self {
            compression: parquet::basic::Compression::SNAPPY,
            batch_size: 10000,
        }
    }

    /// Set compression codec
    pub fn with_compression(mut self, compression: parquet::basic::Compression) -> Self {
        self.compression = compression;
        self
    }

    /// Set batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Write RecordBatches to a Parquet file
    pub fn write(&self, path: &Path, batches: &[RecordBatch]) -> Result<()> {
        if batches.is_empty() {
            return Err(KylinError::Engine("No batches to write".to_string()));
        }

        let file = File::create(path)
            .map_err(|e| KylinError::Engine(format!("Failed to create file: {}", e)))?;

        let props = WriterProperties::builder()
            .set_compression(self.compression)
            .set_max_row_group_size(self.batch_size)
            .build();

        let mut writer = ArrowWriter::try_new(file, batches[0].schema(), Some(props))
            .map_err(|e| KylinError::Engine(format!("Failed to create writer: {}", e)))?;

        for batch in batches {
            writer
                .write(batch)
                .map_err(|e| KylinError::Engine(format!("Failed to write batch: {}", e)))?;
        }

        writer
            .close()
            .map_err(|e| KylinError::Engine(format!("Failed to close writer: {}", e)))?;

        Ok(())
    }

    /// Write a single RecordBatch to a Parquet file
    pub fn write_batch(&self, path: &Path, batch: &RecordBatch) -> Result<()> {
        self.write(path, std::slice::from_ref(batch))
    }
}

impl Default for ParquetWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Write RecordBatches to Parquet file (convenience function)
pub fn write_parquet(path: &Path, batches: &[RecordBatch]) -> Result<()> {
    ParquetWriter::new().write(path, batches)
}

/// Read a Parquet file into RecordBatches
pub fn read_parquet(path: &Path) -> Result<Vec<RecordBatch>> {
    let file = File::open(path)
        .map_err(|e| KylinError::Engine(format!("Failed to open file: {}", e)))?;

    let reader = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| KylinError::Engine(format!("Failed to create reader: {}", e)))?
        .build()
        .map_err(|e| KylinError::Engine(format!("Failed to build reader: {}", e)))?;

    let mut batches = Vec::new();
    for batch in reader {
        let batch = batch.map_err(|e| KylinError::Engine(format!("Failed to read batch: {}", e)))?;
        batches.push(batch);
    }

    Ok(batches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    #[test]
    fn test_write_read_parquet() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
            ],
        )
        .unwrap();

        let path = std::env::temp_dir().join("test.parquet");

        // Write
        write_parquet(&path, &[batch.clone()]).unwrap();

        // Read
        let batches = read_parquet(&path).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 3);

        // Cleanup
        std::fs::remove_file(&path).unwrap();
    }
}
