use std::env;
use std::process;
use ir_eval_rust::IREvalDataset;
use ir_eval_rust::IRMetric;


struct IRArgs {
    ground_truth_qrels_path: String,
    retrieved_qrels_path: String,
    ks: Vec<u32>,
    metrics: Vec<IRMetric>,
}

fn parse_args() -> Result<IRArgs, &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        return Err("Usage: ir_eval_rust <ground_truth_qrels_path> <retrieved_qrels_path> <ks> <metrics>");
    }

    let ground_truth_qrels_path = args[1].clone();
    let retrieved_qrels_path = args[2].clone();
    let ks: Vec<u32> = args[3]
        .split(",")
        .map(|k| k.parse().unwrap())
        .collect();
    let metrics: Vec<IRMetric> = args[4]
        .split(",")
        .map(|m| m.parse().unwrap())
        .collect();
    
    Ok(IRArgs {
        ground_truth_qrels_path,
        retrieved_qrels_path,
        ks,
        metrics,
    })
}

fn main() {
    let ir_eval_args = parse_args();

    if let Err(e) = ir_eval_args {
        eprintln!("Error parsing arguments: {}", e);
        process::exit(1);
    }

    let ir_eval_args = ir_eval_args.unwrap();
    
    let mut ir_eval_dataset =
        IREvalDataset::build_from_disk(
            &ir_eval_args.ground_truth_qrels_path,
            &ir_eval_args.retrieved_qrels_path,
        );
    
    
}
