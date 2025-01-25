use std::collections::HashMap;
use std::str::FromStr;
use std::fmt;

pub enum IRMetric {
    NDCG,
    DCG,
    CG,
    Precision,
    Recall,
    RR,
    Rel
}

impl fmt::Display for IRMetric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRMetric::NDCG => write!(f, "ndcg"),
            IRMetric::DCG => write!(f, "dcg"),
            IRMetric::CG => write!(f, "cg"),
            IRMetric::Precision => write!(f, "precision"),
            IRMetric::Recall => write!(f, "recall"),
            IRMetric::RR => write!(f, "rr"),
            IRMetric::Rel => write!(f, "rel"),
        }
    }
}

impl FromStr for IRMetric {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ndcg" => Ok(IRMetric::NDCG),
            "dcg" => Ok(IRMetric::DCG),
            "cg" => Ok(IRMetric::CG),
            "precision" => Ok(IRMetric::Precision),
            "recall" => Ok(IRMetric::Recall),
            "rr" => Ok(IRMetric::RR),
            "rel" => Ok(IRMetric::Rel),
            _ => Err("Invalid IR metric"),
        }
    }
}

trait Metric {
    fn calculate(
        &self,
        retrieved: &Vec<u32>,
        relevant: &HashMap<u32, f32>,
        ks: &Vec<u32>,
    ) -> Vec<f32>;
}

impl Metric for IRMetric {
    fn calculate(
        &self,
        retrieved: &Vec<u32>,
        relevant: &HashMap<u32, f32>,
        ks: &Vec<u32>,
    ) -> Vec<f32> {
        match self {
            IRMetric::NDCG => ndcg_at_ks(retrieved, relevant, ks),
            IRMetric::DCG => dcg_at_ks(retrieved, relevant, ks),
            IRMetric::CG => cg_at_ks(retrieved, relevant, ks),
            IRMetric::Precision => precision_at_ks(retrieved, relevant, ks),
            IRMetric::Recall => recall_at_ks(retrieved, relevant, ks),
            IRMetric::RR => rr_at_ks(retrieved, relevant, ks),
            IRMetric::Rel => rel_at_ks(retrieved, relevant, ks),
        }
    }
}

fn cg_at_ks(retrieved: &Vec<u32>, relevant: &HashMap<u32, f32>, ks: &Vec<u32>) -> Vec<f32> {
    let mut cg_at_ks = Vec::with_capacity(ks.len());
    let mut cg = 0.0 as f32;
    let mut curr_k_idx = 0 as usize;
    for (idx, doc_id) in retrieved.iter().enumerate() {
        if let Some(rel) = relevant.get(doc_id) {
            cg += *rel;
        }

        if idx + 1 == ks[curr_k_idx] as usize {
            cg_at_ks.push(cg);
            curr_k_idx += 1;
        }
    }
    cg_at_ks
}

fn dcg_at_ks(retrieved: &Vec<u32>, relevant: &HashMap<u32, f32>, ks: &Vec<u32>) -> Vec<f32> {
    let mut dcg_at_ks = Vec::with_capacity(ks.len());
    let mut dcg = 0.0 as f32;
    let mut curr_k_idx = 0 as usize;
    for (idx, doc_id) in retrieved.iter().enumerate() {
        if let Some(rel) = relevant.get(doc_id) {
            dcg += *rel / ((idx as f32 + 2.0).log2());
        }

        if idx + 1 == ks[curr_k_idx] as usize {
            dcg_at_ks.push(dcg);
            curr_k_idx += 1;
        }

        if curr_k_idx == ks.len() {
            break;
        }
    }
    dcg_at_ks
}

fn ndcg_at_ks(retrieved: &Vec<u32>, relevant: &HashMap<u32, f32>, ks: &Vec<u32>) -> Vec<f32> {
    let mut ndcg_at_ks = Vec::with_capacity(ks.len());
    let dcg_at_ks_res = dcg_at_ks(retrieved, relevant, ks);
    let mut sorted_relevant_ids: Vec<(&u32, &f32)> = relevant.iter().collect();
    sorted_relevant_ids.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    // Ideal order (just the document IDs, sorted by relevance)
    let ideal_order: Vec<u32> = sorted_relevant_ids
        .into_iter()
        .map(|(doc_id, _)| *doc_id)
        .collect();

    // Calculate the Ideal DCG (IDCG) at k values
    let idcg_at_ks = dcg_at_ks(&ideal_order, relevant, ks);

    for (dcg, idcg) in dcg_at_ks_res.iter().zip(idcg_at_ks.iter()) {
        if *idcg == 0.0 {
            ndcg_at_ks.push(0.0);
        } else {
            ndcg_at_ks.push(dcg / idcg);
        }
    }
    ndcg_at_ks
}

fn precision_at_ks(retrieved: &Vec<u32>, relevant: &HashMap<u32, f32>, ks: &Vec<u32>) -> Vec<f32> {
    let mut precision_at_ks = Vec::with_capacity(ks.len());
    let mut curr_k_idx = 0 as usize;
    let mut relevant_count = 0;
    for (idx, doc_id) in retrieved.iter().enumerate() {
        if relevant.contains_key(doc_id) {
            relevant_count += 1;
        }

        if idx + 1 == ks[curr_k_idx] as usize {
            precision_at_ks.push(relevant_count as f32 / (idx as f32 + 1.0));
            curr_k_idx += 1;
        }

        if curr_k_idx == ks.len() {
            break;
        }
    }
    precision_at_ks
}

fn recall_at_ks(retrieved: &Vec<u32>, relevant: &HashMap<u32, f32>, ks: &Vec<u32>) -> Vec<f32> {
    let mut recall_at_ks = Vec::with_capacity(ks.len());
    let mut recall = 0.0 as f32;
    let mut curr_k_idx = 0 as usize;
    let mut relevant_count = 0;
    let total_relevant = relevant.len();
    for (idx, doc_id) in retrieved.iter().enumerate() {
        if relevant.contains_key(doc_id) {
            relevant_count += 1;
            recall = relevant_count as f32 / total_relevant as f32;
        }

        if idx + 1 == ks[curr_k_idx] as usize {
            recall_at_ks.push(recall);
            curr_k_idx += 1;
        }

        if curr_k_idx == ks.len() {
            break;
        }
    }
    recall_at_ks
}

fn rr_at_ks(retrieved: &Vec<u32>, relevant: &HashMap<u32, f32>, ks: &Vec<u32>) -> Vec<f32> {
    let mut rr_at_ks = Vec::with_capacity(ks.len());
    let mut rr = 0.0 as f32;
    let mut curr_k_idx = 0 as usize;
    for (idx, doc_id) in retrieved.iter().enumerate() {
        if rr == 0.0 && relevant.contains_key(doc_id) {
            rr = 1.0 / (idx as f32 + 1.0);
            break;
        }

        if idx + 1 == ks[curr_k_idx] as usize {
            rr_at_ks.push(rr);
            curr_k_idx += 1;
        }

        if curr_k_idx == ks.len() {
            break;
        }
    }

    for _ in curr_k_idx..ks.len() {
        rr_at_ks.push(rr);
    }

    rr_at_ks
}

fn rel_at_ks(retrieved: &Vec<u32>, relevant: &HashMap<u32, f32>, ks: &Vec<u32>) -> Vec<f32> {
    let mut rel_at_ks = Vec::with_capacity(ks.len());
    let mut rel = 0.0 as f32;
    let mut curr_k_idx = 0 as usize;
    for (idx, doc_id) in retrieved.iter().enumerate() {
        if relevant.contains_key(doc_id) {
            rel += 1.0;
        }

        if idx + 1 == ks[curr_k_idx] as usize {
            rel_at_ks.push(rel);
            curr_k_idx += 1;
        }

        if curr_k_idx == ks.len() {
            break;
        }
    }

    rel_at_ks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_close_with_epsilon(a: f32, b: f32, epsilon: f32) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_cg_at_ks() {
        let retrieved = vec![1, 2, 3, 4];
        let mut relevant = HashMap::new();
        relevant.insert(1 as u32, 1.0 as f32);
        relevant.insert(3, 1.0);
        relevant.insert(4, 1.0);
        let ks = vec![1, 2, 3, 4];
        let cg_at_ks = cg_at_ks(&retrieved, &relevant, &ks);
        assert_eq!(cg_at_ks, vec![1.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_dcg_at_ks() {
        let retrieved = vec![1, 2, 3, 4];
        let mut relevant = HashMap::new();
        relevant.insert(1 as u32, 1.0 as f32);
        relevant.insert(3, 1.0);
        relevant.insert(4, 1.0);
        let ks = vec![1, 2, 3, 4];
        let dcg_at_ks = dcg_at_ks(&retrieved, &relevant, &ks);
        let expected_dcg_at_ks = vec![1.0, 1.0, 1.5, 1.9306765580733];
        for (dcg, expected_dcg) in dcg_at_ks.iter().zip(expected_dcg_at_ks.iter()) {
            assert!(is_close_with_epsilon(*dcg, *expected_dcg, 0.0001));
        }
    }

    #[test]
    fn test_ndcg_at_ks() {
        let retrieved = vec![1, 2, 3, 4];
        let mut relevant = HashMap::new();
        relevant.insert(1 as u32, 1.0 as f32);
        relevant.insert(3, 1.0);
        relevant.insert(4, 1.0);
        let ks = vec![1, 2, 3, 4];
        let ndcg_at_ks = ndcg_at_ks(&retrieved, &relevant, &ks);
        let expected_ndcg_at_ks = vec![1.0, 0.613147, 0.703918, 0.906025];

        for (ndcg, expected_ndcg) in ndcg_at_ks.iter().zip(expected_ndcg_at_ks.iter()) {
            assert!(is_close_with_epsilon(*ndcg, *expected_ndcg, 0.0001));
        }
    }

    #[test]
    fn test_precision_at_ks() {
        let retrieved = vec![1, 2, 3, 4];
        let mut relevant = HashMap::new();
        relevant.insert(1 as u32, 1.0 as f32);
        relevant.insert(3, 1.0);
        relevant.insert(4, 1.0);
        let ks = vec![1, 2, 3, 4];
        let precision_at_ks = precision_at_ks(&retrieved, &relevant, &ks);
        println!("{:?}", precision_at_ks);
        let expected_precision_at_ks = vec![1.0, 0.5, 0.6666666, 0.75];
        for (precision, expected_precision) in
            precision_at_ks.iter().zip(expected_precision_at_ks.iter())
        {
            assert!(is_close_with_epsilon(
                *precision,
                *expected_precision,
                0.0001
            ));
        }
    }

    #[test]
    fn test_recall_at_ks() {
        let retrieved = vec![1, 2, 3, 4];
        let mut relevant = HashMap::new();
        relevant.insert(1 as u32, 1.0 as f32);
        relevant.insert(3, 1.0);
        relevant.insert(4, 1.0);
        let ks = vec![1, 2, 3, 4];
        let recall_at_ks = recall_at_ks(&retrieved, &relevant, &ks);
        let expected_recall_at_ks = vec![0.33333, 0.33333333, 0.6666666, 1.0];
        for (recall, expected_recall) in recall_at_ks.iter().zip(expected_recall_at_ks.iter()) {
            assert!(is_close_with_epsilon(*recall, *expected_recall, 0.0001));
        }
    }

    #[test]
    fn test_rr_at_ks() {
        let retrieved = vec![1, 2, 3, 4];
        let mut relevant = HashMap::new();
        relevant.insert(3 as u32, 1.0 as f32);
        relevant.insert(4, 1.0);
        let ks = vec![1, 2, 3, 4];
        let rr_at_ks = rr_at_ks(&retrieved, &relevant, &ks);
        let expected_rr_at_ks = vec![0.0, 0.0, 0.33333333, 0.33333333];
        for (rr, expected_rr) in rr_at_ks.iter().zip(expected_rr_at_ks.iter()) {
            assert!(is_close_with_epsilon(*rr, *expected_rr, 0.0001));
        }
    }

    #[test]
    fn test_rel_at_ks() {
        let retrieved = vec![1, 2, 3, 4];
        let mut relevant = HashMap::new();
        relevant.insert(1 as u32, 1.0 as f32);
        relevant.insert(3, 1.0);
        relevant.insert(4, 1.0);
        let ks = vec![1, 2, 3, 4];
        let rel_at_ks = rel_at_ks(&retrieved, &relevant, &ks);
        let expected_rel_at_ks = vec![1.0, 1.0, 2.0, 3.0];
        for (rel, expected_rel) in rel_at_ks.iter().zip(expected_rel_at_ks.iter()) {
            assert!(is_close_with_epsilon(*rel, *expected_rel, 0.0001));
        }
    }
}
