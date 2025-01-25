use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub mod metrics;
pub mod qrels;
pub mod aggregates;

pub use metrics::IRMetric;
pub use qrels::IREvalDataset;
pub use aggregates::{AggregateMetrics, aggregate_results};

struct IREvalResults {
    query_results: HashMap<String, HashMap<String, f32>>,
    aggregate_results: HashMap<String, f32>,
    metadata: IREvalMetadata,
}

struct IREvalMetadata {
    run_id: String,
    start_time: u64,
    end_time: u64,
    elapsed_time: u64,
    num_queries: u32,
    num_docs: u32,
    num_relevant_docs: u32,
    num_retrieved_docs: u32,
}

// pub fn evaluate(dataset: &IREvalDataset,metrics: Vec<IRMetric>,ks: Vec<u32>,run_id: Option<String>) -> IREvalResults {
//     let start_time = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs();
//     let mut query_results = HashMap::new();
//     let mut aggregate_results = HashMap::new();
//     let mut num_queries = 0;
//     let mut num_docs = 0;
//     let mut num_relevant_docs = 0;
//     let mut num_retrieved_docs = 0;

    
    
// }