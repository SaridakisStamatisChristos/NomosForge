//! nomos: finite institutional rule synthesis and single-state conflict checks.
use nomos_grammar::worlds;
use nomos_parse::parse;
use nomos_synth::synthesize;
use std::{env, fs, process};
fn run(args: &[String]) -> Result<i32, String> {
    if args.len() < 2 {
        return Err("usage: nomos validate|synth|explain|doctor [file]".into());
    }
    match args[1].as_str() {
        "relax" => {
            if args.len() < 3 {
                return Err("usage: nomos relax conflict-world.yaml --show-all-minimal".into());
            }
            let src = fs::read_to_string(&args[2]).map_err(|e| e.to_string())?;
            let case = nomos_conflict::relax::parse_case(&src)?;
            let report =
                nomos_conflict::relax::propose(&case, nomos_conflict::symmetric_single_token())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            Ok(0)
        }
        "doctor" => {
            println!("NomosForge 0.1: finite enumerator only; no SMT/social validity");
            Ok(0)
        }
        "explain" => {
            if args.len() != 3 {
                return Err("usage: nomos explain certificate.json".into());
            }
            let src = fs::read_to_string(&args[2]).map_err(|e| e.to_string())?;
            let cert: nomos_conflict::ConflictCertificate =
                serde_json::from_str(&src).map_err(|e| e.to_string())?;
            nomos_conflict::check_certificate(&cert)?;
            println!(
                "CERTIFICATE_CHECKED: MODEL_INFEASIBLE only for {}",
                cert.schema
            );
            for core in cert.minimal_conflict_cores {
                println!("subset-minimal conflict: {}", core.join(", "));
            }
            Ok(3)
        }
        "conflict" => {
            let cert = nomos_conflict::symmetric_single_token();
            nomos_conflict::check_certificate(&cert)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&cert).map_err(|e| e.to_string())?
            );
            Ok(3)
        }
        "validate" | "synth" | "frontier" => {
            if args.len() < 3 {
                return Err("world YAML file required".into());
            }
            let src = fs::read_to_string(&args[2]).map_err(|e| e.to_string())?;
            let spec = parse(&src)?;
            if args[1] == "validate" {
                println!("VALID: {} ({})", spec.model.id, spec.schema);
                return Ok(0);
            }
            let flags: Vec<u8> = spec
                .verification
                .critical
                .iter()
                .map(|&c| u8::from(c))
                .collect();
            let domain = worlds(
                spec.model.people.count,
                spec.verification.stock.min,
                spec.verification.stock.max,
                spec.verification.need.min,
                spec.verification.need.max,
                &flags,
            )?;
            let grammar = nomos_grammar::grammar(
                &spec.policies.grammar.priority,
                &spec.policies.grammar.reserve,
            )?;
            if args[1] == "frontier" {
                let report = nomos_report::analyze(&domain, &grammar)?;
                println!("{}",serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?);
                return Ok(if report.feasible_count>0 {0} else {3});
            }
            let result = synthesize(&domain, &grammar, None)?;
            let json = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())? + "\n";
            if let Some(pos) = args.iter().position(|a| a == "--json") {
                let path = args.get(pos + 1).ok_or("missing path after --json")?;
                fs::write(path, json).map_err(|e| e.to_string())?;
            } else {
                print!("{json}");
            }
            match result.status.as_str() {
                "BOUNDED_VERIFIED_IN_FINITE_GRAMMAR" => Ok(0),
                "UNREALIZABLE_IN_FINITE_GRAMMAR" => Ok(3),
                "UNKNOWN" => Ok(4),
                _ => Ok(5),
            }
        }
        _ => Err("unknown subcommand".into()),
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    match run(&args) {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("INVALID: {e}");
            process::exit(2)
        }
    }
}
