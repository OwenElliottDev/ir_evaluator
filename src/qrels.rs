use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct IREvalDataset {
    ground_truth_qrels: Vec<HashMap<u32, f32>>,
    retrieved_qrels: Vec<HashMap<u32, f32>>,
    qid_to_idx: HashMap<String, usize>,
    docid_to_int: HashMap<String, u32>,
    ground_truth_qrel_count: usize,
    retrieved_qrel_count: usize,
    doc_count: u32,
}

impl IREvalDataset {
    pub fn new() -> Self {
        IREvalDataset {
            ground_truth_qrels: Vec::new(),
            retrieved_qrels: Vec::new(),
            qid_to_idx: HashMap::new(),
            docid_to_int: HashMap::new(),
            ground_truth_qrel_count: 0,
            retrieved_qrel_count: 0,
            doc_count: 0,
        }
    }

    pub fn build_from_disk(ground_truth_qrels_path: &str, retrieved_qrels_path: &str) -> Self {
        let mut dataset = IREvalDataset::new();
        dataset.load_trec_qrels(ground_truth_qrels_path, true);
        dataset.load_trec_qrels(retrieved_qrels_path, false);
        dataset
    }

    pub fn load_trec_qrels(&mut self, ground_truth_qrels_path: &str, ground_truth: bool) {
        // Load ground truth qrels from file, file is a TREC qrels file
        // Format: query_id(str)\titeration(u32)\tdocument_id(str)\trelevance(f32)

        // Open file
        let file = File::open(ground_truth_qrels_path).expect("Could not open file");
        let reader = BufReader::new(file);

        // Read file line by line
        for line in reader.lines() {
            let line = line.expect("Could not read line");
            let fields: Vec<&str> = line.split_whitespace().collect();
            let qid = fields[0].to_string();
            let docid = fields[2].to_string();
            let rel = fields[3].parse::<f32>().expect("Could not parse relevance");
            if ground_truth {
                self.add_ground_truth_qrel(qid, docid, rel);
            } else {
                self.add_retrieved_qrel(qid, docid, rel);
            }
        }
    }

    pub fn add_ground_truth_qrel(&mut self, qid: String, docid: String, rel: f32) {
        let docid_int = match self.docid_to_int.get(&docid) {
            Some(&docid_int) => docid_int,
            None => {
                self.doc_count += 1;
                self.docid_to_int.insert(docid.clone(), self.doc_count);
                self.doc_count
            }
        };
        let qrels_int = self.qid_to_idx.entry(qid.clone()).or_insert_with(|| {
            self.ground_truth_qrels.push(HashMap::new());
            self.ground_truth_qrel_count += 1;
            self.ground_truth_qrel_count - 1
        });
        self.ground_truth_qrels[*qrels_int].insert(docid_int, rel);
    }

    pub fn add_retrieved_qrel(&mut self, qid: String, docid: String, rel: f32) {
        let docid_int = match self.docid_to_int.get(&docid) {
            Some(&docid_int) => docid_int,
            None => {
                self.doc_count += 1;
                self.docid_to_int.insert(docid.clone(), self.doc_count);
                self.doc_count
            }
        };
        let qrels_int = self.qid_to_idx.entry(qid.clone()).or_insert_with(|| {
            self.retrieved_qrels.push(HashMap::new());
            self.retrieved_qrel_count += 1;
            self.retrieved_qrel_count - 1
        });
        self.retrieved_qrels[*qrels_int].insert(docid_int, rel);
    }

    pub fn add_ground_truth_qrels(&mut self, qid: String, qrels: HashMap<String, f32>) {
        let mut qrels_int = HashMap::new();
        for (docid, rel) in qrels {
            let docid_int = match self.docid_to_int.get(&docid) {
                Some(&docid_int) => docid_int,
                None => {
                    self.doc_count += 1;
                    self.docid_to_int.insert(docid.clone(), self.doc_count);
                    self.doc_count
                }
            };
            qrels_int.insert(docid_int, rel);
        }
        self.qid_to_idx
            .insert(qid.clone(), self.ground_truth_qrel_count);
        self.ground_truth_qrels.push(qrels_int);
        self.ground_truth_qrel_count += 1;
    }

    pub fn add_retrieved_qrels(&mut self, qid: String, qrels: HashMap<String, f32>) {
        let mut qrels_int = HashMap::new();
        for (docid, rel) in qrels {
            let docid_int = match self.docid_to_int.get(&docid) {
                Some(&docid_int) => docid_int,
                None => {
                    self.doc_count += 1;
                    self.docid_to_int.insert(docid.clone(), self.doc_count);
                    self.doc_count
                }
            };
            qrels_int.insert(docid_int, rel);
        }
        self.qid_to_idx
            .insert(qid.clone(), self.retrieved_qrel_count);
        self.retrieved_qrels.push(qrels_int);
        self.retrieved_qrel_count += 1;
    }

    pub fn get_ground_truth_qrels(&self, qid: &str) -> Option<&HashMap<u32, f32>> {
        match self.qid_to_idx.get(qid) {
            Some(&idx) => Some(&self.ground_truth_qrels[idx]),
            None => None,
        }
    }

    pub fn get_docid(&self, docid: &str) -> Option<u32> {
        match self.docid_to_int.get(docid) {
            Some(&docid_int) => Some(docid_int),
            None => None,
        }
    }

    pub fn get_ground_truth_relevance(&self, qid: &str, docid: &str) -> Option<f32> {
        match self.qid_to_idx.get(qid) {
            Some(&idx) => match self.docid_to_int.get(docid) {
                Some(&docid_int) => match self.ground_truth_qrels[idx].get(&docid_int) {
                    Some(&rel) => Some(rel),
                    None => None,
                },
                None => None,
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_ground_truth_qrel() {
        let mut dataset = IREvalDataset::new();
        dataset.add_ground_truth_qrel("1".to_string(), "doc1".to_string(), 1.0);
        dataset.add_ground_truth_qrel("1".to_string(), "doc2".to_string(), 0.0);
        dataset.add_ground_truth_qrel("2".to_string(), "doc1".to_string(), 0.0);
        assert_eq!(dataset.ground_truth_qrel_count, 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.ground_truth_qrels.len(), 2);
        assert_eq!(dataset.ground_truth_qrels[0].len(), 2);
        assert_eq!(dataset.ground_truth_qrels[1].len(), 1);
    }

    #[test]
    fn test_add_retrieved_qrel() {
        let mut dataset = IREvalDataset::new();
        dataset.add_retrieved_qrel("1".to_string(), "doc1".to_string(), 1.0);
        dataset.add_retrieved_qrel("1".to_string(), "doc2".to_string(), 0.0);
        dataset.add_retrieved_qrel("2".to_string(), "doc1".to_string(), 0.0);
        assert_eq!(dataset.retrieved_qrel_count, 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.retrieved_qrels.len(), 2);
        assert_eq!(dataset.retrieved_qrels[0].len(), 2);
        assert_eq!(dataset.retrieved_qrels[1].len(), 1);
    }

    #[test]
    fn test_add_ground_truth_qrels() {
        let mut dataset = IREvalDataset::new();
        let mut qrels = HashMap::new();
        qrels.insert("doc1".to_string(), 1.0);
        qrels.insert("doc2".to_string(), 0.0);
        dataset.add_ground_truth_qrels("1".to_string(), qrels);
        let mut qrels = HashMap::new();
        qrels.insert("doc1".to_string(), 0.0);
        dataset.add_ground_truth_qrels("2".to_string(), qrels);
        assert_eq!(dataset.ground_truth_qrel_count, 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.ground_truth_qrels.len(), 2);
        assert_eq!(dataset.ground_truth_qrels[0].len(), 2);
        assert_eq!(dataset.ground_truth_qrels[1].len(), 1);
    }

    #[test]
    fn test_add_retrieved_qrels() {
        let mut dataset = IREvalDataset::new();
        let mut qrels = HashMap::new();
        qrels.insert("doc1".to_string(), 1.0);
        qrels.insert("doc2".to_string(), 0.0);
        dataset.add_retrieved_qrels("1".to_string(), qrels);
        let mut qrels = HashMap::new();
        qrels.insert("doc1".to_string(), 0.0);
        dataset.add_retrieved_qrels("2".to_string(), qrels);
        println!("{:?}", dataset.retrieved_qrel_count);
        assert_eq!(dataset.retrieved_qrel_count, 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.retrieved_qrels.len(), 2);
        assert_eq!(dataset.retrieved_qrels[0].len(), 2);
        assert_eq!(dataset.retrieved_qrels[1].len(), 1);
    }

    #[test]
    fn test_get_ground_truth_qrels() {
        let mut dataset = IREvalDataset::new();
        let mut qrels = HashMap::new();
        qrels.insert("doc1".to_string(), 1.0);
        qrels.insert("doc2".to_string(), 0.0);
        dataset.add_ground_truth_qrels("1".to_string(), qrels);
        let mut qrels = HashMap::new();
        qrels.insert("doc1".to_string(), 0.0);
        dataset.add_ground_truth_qrels("2".to_string(), qrels);
        assert_eq!(dataset.get_ground_truth_qrels("1").unwrap().len(), 2);
        assert_eq!(dataset.get_ground_truth_qrels("2").unwrap().len(), 1);
        assert_eq!(dataset.get_ground_truth_qrels("3"), None);
    }

    #[test]
    fn test_get_docid() {
        let mut dataset = IREvalDataset::new();
        dataset.add_ground_truth_qrel("1".to_string(), "doc1".to_string(), 1.0);
        dataset.add_ground_truth_qrel("1".to_string(), "doc2".to_string(), 0.0);
        dataset.add_ground_truth_qrel("2".to_string(), "doc1".to_string(), 0.0);
        assert_eq!(dataset.get_docid("doc1").unwrap(), 1);
        assert_eq!(dataset.get_docid("doc2").unwrap(), 2);
        assert_eq!(dataset.get_docid("doc3"), None);
    }

    #[test]
    fn test_get_ground_truth_relevance() {
        let mut dataset = IREvalDataset::new();
        dataset.add_ground_truth_qrel("1".to_string(), "doc1".to_string(), 1.0);
        dataset.add_ground_truth_qrel("1".to_string(), "doc2".to_string(), 0.0);
        dataset.add_ground_truth_qrel("2".to_string(), "doc1".to_string(), 0.0);
        assert_eq!(
            dataset.get_ground_truth_relevance("1", "doc1").unwrap(),
            1.0
        );
        assert_eq!(
            dataset.get_ground_truth_relevance("1", "doc2").unwrap(),
            0.0
        );
        assert_eq!(dataset.get_ground_truth_relevance("1", "doc3"), None);
        assert_eq!(
            dataset.get_ground_truth_relevance("2", "doc1").unwrap(),
            0.0
        );
        assert_eq!(dataset.get_ground_truth_relevance("2", "doc2"), None);
    }
}
