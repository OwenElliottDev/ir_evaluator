use core::panic;
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
    let run_id = match run_id {
        Some(run_id) => run_id,
        None => Uuid::new_v4().to_string(),
    };

    let metrics = match metrics {
        Some(metrics) => metrics,
        None => IRMetric::all_metrics(),
    };

    let ks = match ks {
        Some(ks) => ks,
        None => vec![1, 5, 10, 20, 50, 100, 200, 500, 1000],
    };

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut query_results: HashMap<String, HashMap<String, f32>> = HashMap::new();
    let mut num_queries = 0;
    let mut num_relevant_docs = 0;
    let mut num_retrieved_docs = 0;

    let mut metric_names: Vec<String> = Vec::new();
    for metric in metrics.iter() {
        for k in ks.iter() {
            let metric_name = format!("{}@{}", metric, k);
            metric_names.push(metric_name);
        }
    }

    for query in dataset.ground_truth_qrels.keys() {
        let gt_qrels = match dataset.get_ground_truth_qrels(query) {
            Some(gt_qrels) => gt_qrels,
            None => panic!("No ground truth qrels found for query: {}", query),
        };

        let retrieved_qrels = match dataset.get_retrieved_qrels(query) {
            Some(retrieved_qrels) => retrieved_qrels,
            None => panic!("No retrieved qrels found for query: {}", query),
        };

        let retrieved_ids: Vec<u32> = retrieved_qrels
            .iter()
            .map(|scored_doc| scored_doc.docid)
            .collect();

        for metric in metrics.iter() {
            let metric_results = metric.calculate(&retrieved_ids, gt_qrels, &ks);
            for (idx, score) in metric_results.iter().enumerate() {
                let metric_name = format!("{}@{}", metric, ks[idx]);
                let query_result = query_results.entry(query.clone()).or_insert(HashMap::new());
                query_result.insert(metric_name, *score);
            }
        }

        num_queries += 1;
    }

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
            num_queries,
            num_docs: dataset.doc_count,
            num_relevant_docs,
            num_retrieved_docs,
        },
    }
}
