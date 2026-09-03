use crate::IREvalMetadata;
use crate::IREvalResults;
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AggregatedIREvalResults {
    pub metrics: Vec<String>,
    pub aggregate_results: HashMap<String, f32>,
    pub metadata: IREvalMetadata,
}

impl AggregatedIREvalResults {
    pub fn write_to_json_file(&self, file_path: &str, pretty: bool) -> Result<(), std::io::Error> {
        let json_str = if pretty {
            serde_json::to_string_pretty(self)
        } else {
            serde_json::to_string(self)
        }?;
        std::fs::write(file_path, json_str)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AggregationType {
    Mean,
    Median,
    Max,
    Min,
    Sum,
}

impl fmt::Display for AggregationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AggregationType::Mean => write!(f, "mean"),
            AggregationType::Median => write!(f, "median"),
            AggregationType::Max => write!(f, "max"),
            AggregationType::Min => write!(f, "min"),
            AggregationType::Sum => write!(f, "sum"),
        }
    }
}

fn compute_aggregations(
    query_results: HashMap<String, HashMap<String, f32>>,
    aggregations: Vec<AggregationType>,
) -> HashMap<String, f32> {
    let mut aggregate_results = HashMap::new();

    // aggregate each metric across all queries and prefix the metric name with the aggregation type
    // structure of query_results: HashMap<query_id, HashMap<metric_name, metric_value>>

    for metric in query_results.values().next().unwrap().keys() {
        for aggregation in &aggregations {
            let mut metric_values: Vec<f32> = Vec::new();
            for query_result in query_results.values() {
                let metric_value = query_result.get(metric).unwrap();
                metric_values.push(*metric_value);
            }

            let aggregate_value = match aggregation {
                AggregationType::Mean => {
                    metric_values.iter().sum::<f32>() / metric_values.len() as f32
                }
                AggregationType::Median => {
                    metric_values.sort_by(|a, b| a.total_cmp(b));
                    let mid = metric_values.len() / 2;
                    if metric_values.len().is_multiple_of(2) {
                        (metric_values[mid - 1] + metric_values[mid]) / 2.0
                    } else {
                        metric_values[mid]
                    }
                }
                AggregationType::Max => *metric_values
                    .iter()
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap(),
                AggregationType::Min => *metric_values
                    .iter()
                    .min_by(|a, b| a.total_cmp(b))
                    .unwrap(),
                AggregationType::Sum => metric_values.iter().sum(),
            };

            let aggregation_metric_name = format!("{}_{}", aggregation, metric);
            aggregate_results.insert(aggregation_metric_name, aggregate_value);
        }
    }

    aggregate_results
}

pub fn aggregate_results(
    results: IREvalResults,
    aggregations: Option<Vec<AggregationType>>,
) -> AggregatedIREvalResults {
    let aggregations = aggregations.unwrap_or_else(|| vec![AggregationType::Mean]);

    let query_results = results.query_results;

    let aggregate_results = compute_aggregations(query_results, aggregations);

    AggregatedIREvalResults {
        metrics: results.metrics.clone(),
        aggregate_results,
        metadata: results.metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_aggregations() {
        let mut query_results = HashMap::new();
        let mut query_result = HashMap::new();
        query_result.insert("metric1".to_string(), 1.0);
        query_result.insert("metric2".to_string(), 2.0);
        query_results.insert("query1".to_string(), query_result);

        let mut query_result = HashMap::new();
        query_result.insert("metric1".to_string(), 3.0);
        query_result.insert("metric2".to_string(), 4.0);
        query_results.insert("query2".to_string(), query_result);

        let aggregations = vec![AggregationType::Mean, AggregationType::Sum];

        let aggregate_results = compute_aggregations(query_results, aggregations);

        assert_eq!(aggregate_results.get("mean_metric1").unwrap(), &2.0);
        assert_eq!(aggregate_results.get("mean_metric2").unwrap(), &3.0);
        assert_eq!(aggregate_results.get("sum_metric1").unwrap(), &4.0);
        assert_eq!(aggregate_results.get("sum_metric2").unwrap(), &6.0);
    }

    #[test]
    fn test_aggregate_results() {
        let mut query_results = HashMap::new();
        let mut query_result = HashMap::new();
        query_result.insert("metric1".to_string(), 1.0);
        query_result.insert("metric2".to_string(), 2.0);
        query_results.insert("query1".to_string(), query_result);

        let mut query_result = HashMap::new();
        query_result.insert("metric1".to_string(), 3.0);
        query_result.insert("metric2".to_string(), 4.0);
        query_results.insert("query2".to_string(), query_result);

        let ir_eval_results = IREvalResults {
            metrics: vec!["metric1".to_string(), "metric2".to_string()],
            query_results,
            metadata: IREvalMetadata {
                start_time: 0,
                end_time: 0,
                elapsed_time: 0,
                run_id: "test".to_string(),
                num_queries: 0,
                num_docs: 0,
                num_relevant_docs: 0,
                num_retrieved_docs: 0,
            },
        };

        let aggregations = vec![AggregationType::Mean, AggregationType::Sum];

        let aggregate_results = aggregate_results(ir_eval_results, Some(aggregations));

        assert_eq!(
            aggregate_results.metrics,
            vec!["metric1".to_string(), "metric2".to_string()]
        );
        assert_eq!(aggregate_results.metadata.run_id, "test");
        assert_eq!(
            aggregate_results
                .aggregate_results
                .get("mean_metric1")
                .unwrap(),
            &2.0
        );
        assert_eq!(
            aggregate_results
                .aggregate_results
                .get("mean_metric2")
                .unwrap(),
            &3.0
        );
        assert_eq!(
            aggregate_results
                .aggregate_results
                .get("sum_metric1")
                .unwrap(),
            &4.0
        );
        assert_eq!(
            aggregate_results
                .aggregate_results
                .get("sum_metric2")
                .unwrap(),
            &6.0
        );
    }
}
