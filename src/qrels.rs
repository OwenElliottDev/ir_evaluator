use memmap2::MmapOptions;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::str;

#[derive(Debug, PartialEq)]
pub struct ScoredDoc {
    pub docid: u32,
    pub score: f32,
}

impl Eq for ScoredDoc {}

impl Ord for ScoredDoc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(Ordering::Less)
            .reverse()
    }
}

impl PartialOrd for ScoredDoc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct IREvalDataset {
    pub queries: Vec<String>,
    pub ground_truth_qrels: HashMap<String, HashMap<u32, f32>>,
    pub retrieved_qrels: HashMap<String, Vec<ScoredDoc>>,
    docid_to_int: HashMap<String, u32>,
    pub doc_count: u32,
}

impl Default for IREvalDataset {
    fn default() -> Self {
        Self::new()
    }
}

impl IREvalDataset {
    pub fn new() -> Self {
        IREvalDataset {
            queries: Vec::new(),
            ground_truth_qrels: HashMap::new(),
            retrieved_qrels: HashMap::new(),
            docid_to_int: HashMap::new(),
            doc_count: 0,
        }
    }

    pub fn get_ground_truth_qrels_count(&self) -> usize {
        self.ground_truth_qrels.len()
    }

    pub fn get_retrieved_qrels_count(&self) -> usize {
        self.retrieved_qrels.len()
    }

    pub fn build_from_disk(ground_truth_qrels_path: &str, retrieved_qrels_path: &str) -> Self {
        let mut dataset = IREvalDataset::new();
        dataset.load_trec_qrels(ground_truth_qrels_path, true);
        dataset.load_trec_qrels(retrieved_qrels_path, false);
        dataset
    }

    pub fn load_trec_qrels(&mut self, path: &str, ground_truth: bool) {
        // Memory-map the file.
        let file = File::open(path).expect("Could not open file");
        let mmap = unsafe { MmapOptions::new().map(&file).expect("Could not mmap file") };
        let data = str::from_utf8(&mmap).expect("Invalid UTF-8");

        // Collect lines then parse in parallel.
        let lines: Vec<&str> = data.lines().collect();
        let results: Vec<(String, String, f32)> = lines
            .par_iter()
            .map(|line| {
                let mut fields = line.split_whitespace();
                let qid = fields.next().unwrap().to_string();
                fields.next(); // skip iteration field
                let docid = fields.next().unwrap().to_string();
                let rel = fields
                    .next()
                    .unwrap()
                    .parse::<f32>()
                    .expect("Could not parse relevance");
                (qid, docid, rel)
            })
            .collect();

        for (qid, docid, rel) in results {
            if ground_truth {
                self.add_ground_truth_qrel(qid, docid, rel);
            } else {
                self.add_retrieved_qrel(qid, docid, rel);
            }
        }
    }

    /// Return the internal integer id for `docid`, assigning a fresh one on first sight.
    fn intern_docid(&mut self, docid: &str) -> u32 {
        if let Some(&docid_int) = self.docid_to_int.get(docid) {
            return docid_int;
        }
        self.doc_count += 1;
        self.docid_to_int.insert(docid.to_string(), self.doc_count);
        self.doc_count
    }

    pub fn add_ground_truth_qrel(&mut self, qid: String, docid: String, rel: f32) {
        let docid_int = self.intern_docid(&docid);
        self.ground_truth_qrels
            .entry(qid)
            .or_default()
            .insert(docid_int, rel);
    }

    pub fn add_retrieved_qrel(&mut self, qid: String, docid: String, rel: f32) {
        let docid_int = self.intern_docid(&docid);
        let scored_docs = self.retrieved_qrels.entry(qid).or_default();

        let new_doc = ScoredDoc {
            docid: docid_int,
            score: rel,
        };
        if let Err(pos) = scored_docs.binary_search(&new_doc) {
            scored_docs.insert(pos, new_doc);
        }
    }

    pub fn add_ground_truth_qrels(&mut self, qid: String, qrels: HashMap<String, f32>) {
        let scored_docs = qrels
            .into_iter()
            .map(|(docid, rel)| (self.intern_docid(&docid), rel))
            .collect();
        self.ground_truth_qrels.insert(qid, scored_docs);
    }

    pub fn add_retrieved_qrels(&mut self, qid: String, qrels: HashMap<String, f32>) {
        let mut scored_docs: Vec<ScoredDoc> = qrels
            .into_iter()
            .map(|(docid, rel)| ScoredDoc {
                docid: self.intern_docid(&docid),
                score: rel,
            })
            .collect();
        scored_docs.sort();
        self.retrieved_qrels.insert(qid, scored_docs);
    }

    pub fn get_ground_truth_qrels(&self, qid: &str) -> Option<&HashMap<u32, f32>> {
        self.ground_truth_qrels.get(qid)
    }

    pub fn get_retrieved_qrels(&self, qid: &str) -> Option<&Vec<ScoredDoc>> {
        self.retrieved_qrels.get(qid)
    }

    pub fn get_internal_docid(&self, docid: &str) -> Option<u32> {
        self.docid_to_int.get(docid).copied()
    }

    pub fn get_ground_truth_relevance(&self, qid: &str, docid: &str) -> Option<f32> {
        let docid_int = self.docid_to_int.get(docid)?;
        self.ground_truth_qrels.get(qid)?.get(docid_int).copied()
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
        assert_eq!(dataset.get_ground_truth_qrels_count(), 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.ground_truth_qrels.len(), 2);
        assert_eq!(dataset.ground_truth_qrels[&"1".to_string()].len(), 2);
        assert_eq!(dataset.ground_truth_qrels[&"2".to_string()].len(), 1);
    }

    #[test]
    fn test_add_retrieved_qrel() {
        let mut dataset = IREvalDataset::new();
        dataset.add_retrieved_qrel("1".to_string(), "doc1".to_string(), 1.0);
        dataset.add_retrieved_qrel("1".to_string(), "doc2".to_string(), 0.0);
        dataset.add_retrieved_qrel("2".to_string(), "doc1".to_string(), 0.0);
        assert_eq!(dataset.get_retrieved_qrels_count(), 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.retrieved_qrels.len(), 2);
        assert_eq!(dataset.retrieved_qrels[&"1".to_string()].len(), 2);
        assert_eq!(dataset.retrieved_qrels[&"2".to_string()].len(), 1);
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
        assert_eq!(dataset.get_ground_truth_qrels_count(), 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.ground_truth_qrels.len(), 2);
        assert_eq!(dataset.ground_truth_qrels[&"1".to_string()].len(), 2);
        assert_eq!(dataset.ground_truth_qrels[&"2".to_string()].len(), 1);
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
        assert_eq!(dataset.get_retrieved_qrels_count(), 2);
        assert_eq!(dataset.doc_count, 2);
        assert_eq!(dataset.retrieved_qrels.len(), 2);
        assert_eq!(dataset.retrieved_qrels[&"1".to_string()].len(), 2);
        assert_eq!(dataset.retrieved_qrels[&"2".to_string()].len(), 1);
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
        assert_eq!(dataset.get_internal_docid("doc1").unwrap(), 1);
        assert_eq!(dataset.get_internal_docid("doc2").unwrap(), 2);
        assert_eq!(dataset.get_internal_docid("doc3"), None);
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
