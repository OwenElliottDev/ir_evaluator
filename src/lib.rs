pub mod aggregates;
mod evaluator;
pub mod metrics;
pub mod qrels;

pub use metrics::IRMetric;
pub use qrels::IREvalDataset;
// pub use aggregates::{AggregateMetrics, aggregate_results};
pub use evaluator::{evaluate, IREvalMetadata, IREvalResults};
