use ir_eval_rust::aggregate_results;
use ir_eval_rust::evaluate;
use ir_eval_rust::IREvalDataset;
use ir_eval_rust::IRMetric;
use std::env;
use std::process;
use std::time::Instant;

struct IRArgs {
    ground_truth_qrels_path: String,
    retrieved_qrels_path: String,
    ks: Option<Vec<u32>>,
    metrics: Option<Vec<IRMetric>>,
}

fn parse_args() -> Result<IRArgs, &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 5 {
        return Err(
            "Usage: ir_eval_rust <ground_truth_qrels_path> <retrieved_qrels_path> <ks> <metrics>",
        );
    }

    let ground_truth_qrels_path = args[1].clone();
    let retrieved_qrels_path = args[2].clone();
    let ks: Option<Vec<u32>> = args.get(3).map(|ks| ks.split(",").map(|k| k.parse().unwrap()).collect());

    let metrics: Option<Vec<IRMetric>> = args.get(4).map(|metrics| metrics.split(",").map(|m| m.parse().unwrap()).collect());

    Ok(IRArgs {
        ground_truth_qrels_path,
        retrieved_qrels_path,
        ks,
        metrics,
    })
}

fn main() {
    let process_start_time = Instant::now();

    let ir_eval_args = parse_args();

    if let Err(e) = ir_eval_args {
        eprintln!("Error parsing arguments: {}", e);
        process::exit(1);
    }

    let ir_eval_args = ir_eval_args.unwrap();

    let dataset_creation_start_time = Instant::now();
    let ir_eval_dataset = IREvalDataset::build_from_disk(
        &ir_eval_args.ground_truth_qrels_path,
        &ir_eval_args.retrieved_qrels_path,
    );

    println!(
        "Dataset creation completed in {:?}",
        dataset_creation_start_time.elapsed()
    );

    let eval_start_time = Instant::now();
    let ir_eval_results = evaluate(
        &ir_eval_dataset,
        ir_eval_args.metrics,
        ir_eval_args.ks,
        None,
    );

    println!("Evaluation completed in {:?}", eval_start_time.elapsed());

    let output_write_start_time = Instant::now();
    let output_file_path = "results.json";
    ir_eval_results
        .write_to_json_file(output_file_path, true)
        .unwrap();

    println!(
        "Output writing completed in {:?}",
        output_write_start_time.elapsed()
    );

    let aggregate_results_start_time = Instant::now();
    let aggregated_results = aggregate_results(ir_eval_results, None);
    let output_file_path = "aggregated_results.json";
    aggregated_results
        .write_to_json_file(output_file_path, true)
        .unwrap();

    println!(
        "Aggregated results completed in {:?}",
        aggregate_results_start_time.elapsed()
    );

    let process_end_time = process_start_time.elapsed();

    println!("Process completed in {:?}", process_end_time);
}
