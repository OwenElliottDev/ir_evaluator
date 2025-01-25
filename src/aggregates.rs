use std::collections::HashMap;
use super::IRMetric;


pub struct AggregateResults {
    mean: HashMap<String, f32>,
    median: HashMap<String, f32>,
}

pub fn aggregate_results(results: &Vec<HashMap<String, f32>>, metrics: &Vec<IRMetric>) -> AggregateResults {
    let mut mean = HashMap::new();
    let mut median = HashMap::new();

    for (metric, _) in results[0].iter() {
        let mut metric_values: Vec<f32> = results
            .iter()
            .map(|result| result.get(metric).unwrap())
            .cloned()
            .collect();
        metric_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let metric_mean = metric_values.iter().sum::<f32>() / metric_values.len() as f32;
        let metric_median = if metric_values.len() % 2 == 0 {
            let mid = metric_values.len() / 2;
            (metric_values[mid - 1] + metric_values[mid]) / 2.0
        } else {
            metric_values[metric_values.len() / 2]
        };

        mean.insert(metric.clone(), metric_mean);
        median.insert(metric.clone(), metric_median);
    }

    AggregateResults { mean, median }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_results() {
        let results = vec![
            [("ndcg@10".to_string(), 0.5), ("rr@10".to_string(), 0.25)]
                .iter()
                .cloned()
                .collect(),
            [("ndcg@10".to_string(), 0.75), ("rr@10".to_string(), 0.5)]
                .iter()
                .cloned()
                .collect(),
            [("ndcg@10".to_string(), 0.25), ("rr@10".to_string(), 0.75)]
                .iter()
                .cloned()
                .collect(),
        ];

        let aggregate_results = aggregate_results(&results);

        assert_eq!(aggregate_results.mean.get("ndcg@10").unwrap(), &0.5);
        assert_eq!(aggregate_results.median.get("ndcg@10").unwrap(), &0.5);
        assert_eq!(aggregate_results.mean.get("mrr@10").unwrap(), &0.5);
        assert_eq!(aggregate_results.median.get("mrr@10").unwrap(), &0.5);
    }
}