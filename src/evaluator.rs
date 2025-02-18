use core::panic;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::metrics::IRMetric;
use crate::metrics::Metric;
use crate::qrels::IREvalDataset;

#[derive(Serialize, Deserialize, Debug)]
pub struct IREvalResults {
    pub metrics: Vec<String>,
    pub query_results: HashMap<String, HashMap<String, f32>>,
    pub metadata: IREvalMetadata,
}

impl IREvalResults {
    pub fn serialize_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn serialize_to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn write_to_json_file(&self, file_path: &str, pretty: bool) -> Result<(), std::io::Error> {
        let json_str = match pretty {
            true => self.serialize_to_pretty_json(),
            false => self.serialize_to_json(),
        }?;
        std::fs::write(file_path, json_str)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IREvalMetadata {
    pub run_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub elapsed_time: u64,
    pub num_queries: u32,
    pub num_docs: u32,
    pub num_relevant_docs: u32,
    pub num_retrieved_docs: u32,
}

pub fn evaluate(
    dataset: &IREvalDataset,
    metrics: Option<Vec<IRMetric>>,
    ks: Option<Vec<u32>>,
    run_id: Option<String>,
) -> IREvalResults {
    // Defaults
    let run_id = run_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let metrics = metrics.unwrap_or_else(IRMetric::all_metrics);
    let ks = ks.unwrap_or_else(|| vec![1, 5, 10, 20, 50, 100, 200, 500, 1000]);

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Precompute metric names for metadata
    let metric_names: Vec<String> = metrics
        .iter()
        .flat_map(|metric| ks.iter().map(move |k| format!("{}@{}", metric, k)))
        .collect();

    // Collect query keys and parallelize per-query evaluation
    let queries: Vec<String> = dataset.ground_truth_qrels.keys().cloned().collect();
    let query_results_vec: Vec<(String, HashMap<String, f32>)> = queries
        .par_iter()
        .map(|query| {
            let gt_qrels = dataset
                .get_ground_truth_qrels(query)
                .unwrap_or_else(|| panic!("No ground truth qrels for query: {}", query));
            let retrieved_qrels = dataset
                .get_retrieved_qrels(query)
                .unwrap_or_else(|| panic!("No retrieved qrels for query: {}", query));

            let retrieved_ids: Vec<u32> = retrieved_qrels
                .iter()
                .map(|scored_doc| scored_doc.docid)
                .collect();

            let mut query_result = HashMap::new();
            for metric in &metrics {
                let metric_results = metric.calculate(&retrieved_ids, gt_qrels, &ks);
                for (idx, score) in metric_results.iter().enumerate() {
                    let metric_name = format!("{}@{}", metric, ks[idx]);
                    query_result.insert(metric_name, *score);
                }
            }
            (query.clone(), query_result)
        })
        .collect();

    let query_results: HashMap<String, HashMap<String, f32>> =
        query_results_vec.into_iter().collect();

    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let elapsed_time = end_time - start_time;

    IREvalResults {
        metrics: metric_names,
        query_results,
        metadata: IREvalMetadata {
            run_id,
            start_time,
            end_time,
            elapsed_time,
            num_queries: queries.len() as u32,
            num_docs: dataset.doc_count,
            num_relevant_docs: 0,  // Update if available
            num_retrieved_docs: 0, // Update if available
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qrels::IREvalDataset;

    #[test]
    fn test_evaluate() {
        let mut dataset = IREvalDataset::new();

        dataset.add_ground_truth_qrel("q1".to_string(), "1".to_string(), 1.0);
        dataset.add_ground_truth_qrel("q1".to_string(), "2".to_string(), 1.0);
        dataset.add_retrieved_qrel("q1".to_string(), "1".to_string(), 1.0);
        dataset.add_retrieved_qrel("q1".to_string(), "2".to_string(), 0.5);

        let metrics = vec![IRMetric::Precision, IRMetric::Recall];
        let ks = vec![1, 5, 10];
        let run_id = Some("test".to_string());

        let ir_eval_results = evaluate(&dataset, Some(metrics), Some(ks), run_id);
        assert_eq!(ir_eval_results.metrics.len(), 6);
        assert_eq!(ir_eval_results.query_results.len(), 1);
        assert_eq!(ir_eval_results.metadata.num_queries, 1);
        assert_eq!(ir_eval_results.metadata.num_docs, 2);
    }
}
