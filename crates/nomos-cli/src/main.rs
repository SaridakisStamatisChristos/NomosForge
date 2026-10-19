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
        "simulate" => {
            let path = args.get(2).ok_or("transition JSON file required")?;
            let input = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let model: nomos_verify::transitions::TransitionModel =
                serde_json::from_str(&input).map_err(|e| e.to_string())?;
            let report = nomos_verify::transitions::simulate(&model)?;
            nomos_verify::transitions::check_trace(&model, &report)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            Ok(if report.status == "BOUNDED_TRACE_CHECKED" {
                0
            } else {
                3
            })
        }
        "behavior" => {
            let path = args.get(2).ok_or("behavior scenario JSON required")?;
            let input = fs::read_to_string(path).map_err(|e| e.to_string())?;
            let model: nomos_verify::behavior::BehaviorModel =
                serde_json::from_str(&input).map_err(|e| e.to_string())?;
            let report = nomos_verify::behavior::enumerate(&model)?;
            nomos_verify::behavior::check_report(&model, &report)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            Ok(0)
        }
        "corpus" => {
            let worlds = nomos_grammar::reference_worlds();
            for policy in nomos_grammar::reference_grammar() {
                for world in &worlds {
                    let allocation = nomos_execute::allocate(world, policy)?;
                    let violated = nomos_execute::violation(world, &allocation);
                    println!(
                        "{}",
                        serde_json::json!({
                            "policy": policy, "world": world,
                            "allocation": allocation, "violation": violated
                        })
                    );
                }
            }
            Ok(0)
        }
        "verify" => {
            if args.len() < 4 || args[3] != "--full-domain" {
                return Err("usage: nomos verify report.json --full-domain [--model world.nomos.yaml]".into());
            }
            let model_path = if args.len() == 4 {
                "examples/water-allocation/world.nomos.yaml"
            } else if args.len() == 6 && args[4] == "--model" {
                &args[5]
            } else {
                return Err("unsupported verify arguments".into());
            };
            let source = fs::read_to_string(model_path).map_err(|e| e.to_string())?;
            let spec = parse(&source)?;
            let report_source = fs::read_to_string(&args[2]).map_err(|e| e.to_string())?;
            let supplied: nomos_synth::SynthesisReport =
                serde_json::from_str(&report_source).map_err(|e| e.to_string())?;
            let flags: Vec<u8> = spec.verification.critical.iter().map(|&c| u8::from(c)).collect();
            let domain = worlds(
                spec.model.people.count, spec.verification.stock.min, spec.verification.stock.max,
                spec.verification.need.min, spec.verification.need.max, &flags
            )?;
            let programs = nomos_grammar::grammar(&spec.policies.grammar.priority, &spec.policies.grammar.reserve)?;
            let recomputed = synthesize(&domain, &programs, None)?;
            if supplied != recomputed {
                return Err("reported policy, witness transcript, status or domain count differs from recomputation".into());
            }
            if let Some(policy) = supplied.policy {
                if nomos_verify::check_all(&domain, policy)?.is_some() {
                    return Err("supplied policy is not valid across the declared finite domain".into());
                }
            }
            println!("CERTIFICATE_CHECKED: {} for {} enumerated states of {}", supplied.status, domain.len(), model_path);
            Ok(if supplied.status == "BOUNDED_VERIFIED_IN_FINITE_GRAMMAR" {0} else {3})
        },
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
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
                );
                return Ok(if report.feasible_count > 0 { 0 } else { 3 });
            }
            // Accept only the explicit enumerative backend; unknown flags must not be silently ignored.
            let mut pos = 3;
            let mut saw_json = false;
            let mut saw_engine = false;
            while pos < args.len() {
                match args[pos].as_str() {
                    "--engine" if !saw_engine && pos + 1 < args.len() && args[pos + 1] == "enumerate" => {
                        saw_engine = true;
                        pos += 2;
                    }
                    "--json" if !saw_json && pos + 1 < args.len() => {
                        saw_json = true;
                        pos += 2;
                    }
                    _ => return Err(format!("unsupported argument to synth: {}", args[pos])),
                }
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
