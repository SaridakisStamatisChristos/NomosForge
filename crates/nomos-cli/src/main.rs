//! nomos: finite institutional rule synthesis, bounded evidence only.
use nomos_grammar::worlds;
use nomos_parse::parse;
use nomos_synth::synthesize;
use std::{env, fs, process};
fn run(args: &[String]) -> Result<i32, String> {
    if args.len() < 2 {
        return Err("usage: nomos validate|synth|doctor [world.nomos.yaml]".into());
    }
    match args[1].as_str() {
        "doctor" => {
            println!("NomosForge 0.1: finite enumerator only; no SMT/social validity");
            Ok(0)
        }
        "validate" | "synth" => {
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
