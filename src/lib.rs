mod aggregates;
mod evaluator;
mod metrics;
mod qrels;

pub use aggregates::{aggregate_results, AggregatedIREvalResults};
pub use evaluator::{evaluate, IREvalMetadata, IREvalResults};
pub use metrics::IRMetric;
pub use qrels::IREvalDataset;
