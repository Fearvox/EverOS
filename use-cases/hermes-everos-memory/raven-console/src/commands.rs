use crate::adapters::{hermes, memory, sc, verify};
use crate::audit;
use crate::context::Context;
use crate::model::{DoctorCheck, DoctorReport, Verdict};
use crate::{output, receipt, repl, research, snapshot, tui, RavenResult};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "raven")]
#[command(about = "Raven v1 local-first EverOS operating console")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print local packet, remote gate, memory, and watchlist status.
    Status,
    /// Start the terminal console view.
    Tui,
    /// Start the slash-command REPL.
    Repl,
    /// Send a bounded prompt through Hermes.
    Chat {
        #[command(subcommand)]
        command: ChatCommand,
    },
    /// Show or export owner packet material.
    Packet {
        #[command(subcommand)]
        command: PacketCommand,
    },
    /// Query the EverOS memory bridge.
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    /// Show agent lane status.
    Agents {
        #[command(subcommand)]
        command: Option<AgentsCommand>,
    },
    /// Show hard gates and stop conditions.
    Gates,
    /// Inspect bounded Raven v2 research lanes and packets.
    Research {
        #[command(subcommand)]
        command: ResearchCommand,
    },
    /// Show saved receipts or configured verification runs.
    Runs {
        #[command(subcommand)]
        command: RunsCommand,
    },
    /// Inspect Superconductor session/worktree state.
    Sc {
        #[command(subcommand)]
        command: Option<ScCommand>,
    },
    /// Execute local Raven run commands.
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    /// Check local dependencies and bridge availability.
    Doctor,
    /// Audit native terminal UX and public-safety discipline.
    NativeAudit,
}

#[derive(Subcommand)]
pub enum PacketCommand {
    /// Show the current owner-readable packet summary.
    Show,
    /// Export a sanitized owner packet.
    Export {
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum MemoryCommand {
    /// Check EverOS memory-provider health.
    Health,
    /// Search through the EverOS provider bridge.
    Search { query: Vec<String> },
}

#[derive(Subcommand)]
pub enum ChatCommand {
    /// Send one prompt through Hermes and print the sanitized turn.
    Send {
        /// Override the Hermes process working directory.
        #[arg(long)]
        cwd: Option<PathBuf>,
        /// Write a sanitized chat receipt to a path, or print it with "-".
        #[arg(long)]
        receipt: Option<String>,
        /// Save a sanitized chat receipt under raven/.local-runs/.
        #[arg(long)]
        save: bool,
        prompt: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum AgentsCommand {
    /// List agent/watch lanes.
    List,
}

#[derive(Subcommand)]
pub enum RunsCommand {
    /// List saved receipts or configured verification commands.
    List,
}

#[derive(Subcommand)]
pub enum ScCommand {
    /// Show the full Superconductor report.
    All,
    /// Check the Superconductor socket and API version.
    Status,
    /// List active Superconductor chat sessions.
    Sessions,
    /// List enabled Superconductor providers.
    Providers,
    /// Show current Superconductor worktree status.
    Worktree,
}

#[derive(Subcommand)]
pub enum ResearchCommand {
    /// List bounded v2 research lanes.
    Lanes,
    /// Render one lane as a live-gate-calibrated decision packet.
    Packet {
        lane: String,
        #[arg(long)]
        output: Option<String>,
    },
    /// Check whether architecture synthesis has enough packet evidence.
    Synthesize {
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum RunCommand {
    /// Verify the local Raven packet gates.
    Verify {
        #[arg(long)]
        receipt: Option<String>,
        #[arg(long)]
        save: bool,
    },
}

pub fn execute(cli: Cli, ctx: &Context) -> RavenResult<()> {
    match cli.command.unwrap_or(Commands::Status) {
        Commands::Status => {
            let snapshot = snapshot::build(ctx);
            if cli.json {
                output::json(&snapshot)
            } else {
                output::status(&snapshot);
                Ok(())
            }
        }
        Commands::Tui => tui::run(ctx),
        Commands::Repl => repl::run(ctx),
        Commands::Chat { command } => match command {
            ChatCommand::Send {
                cwd,
                receipt: receipt_target,
                save,
                prompt,
            } => run_chat_command(ctx, cli.json, cwd, receipt_target, save, &prompt.join(" ")),
        },
        Commands::Packet { command } => match command {
            PacketCommand::Show => {
                let snapshot = snapshot::build(ctx);
                if cli.json {
                    output::json(&snapshot.packet)
                } else {
                    output::packet(&snapshot);
                    Ok(())
                }
            }
            PacketCommand::Export { output: target } => {
                let snapshot = snapshot::build(ctx);
                if cli.json {
                    output::json(&snapshot)
                } else {
                    output::write_text(
                        target.as_deref(),
                        &output::packet_export_markdown(&snapshot),
                    )
                }
            }
        },
        Commands::Memory { command } => match command {
            MemoryCommand::Health => {
                let snapshot = snapshot::build(ctx);
                if cli.json {
                    output::json(&snapshot.memory)
                } else {
                    output::memory_health(&snapshot);
                    Ok(())
                }
            }
            MemoryCommand::Search { query } => {
                let result = memory::search(ctx, &query.join(" "));
                if cli.json {
                    output::json(&result)
                } else {
                    output::memory_search(&result);
                    Ok(())
                }
            }
        },
        Commands::Agents { command: _ } => {
            let snapshot = snapshot::build(ctx);
            if cli.json {
                output::json(&snapshot.agents)
            } else {
                output::agents(&snapshot);
                Ok(())
            }
        }
        Commands::Gates => {
            let snapshot = snapshot::build(ctx);
            if cli.json {
                output::json(&serde_json::json!({
                    "remote": snapshot.remote_gates,
                    "local": snapshot.local_gates,
                }))
            } else {
                output::gates(&snapshot);
                Ok(())
            }
        }
        Commands::Research { command } => match command {
            ResearchCommand::Lanes => {
                let lanes = research::list_lanes();
                if cli.json {
                    output::json(&lanes)
                } else {
                    output::research_lanes(&lanes);
                    Ok(())
                }
            }
            ResearchCommand::Packet {
                lane,
                output: target,
            } => {
                let snapshot = snapshot::build(ctx);
                let packet = research::packet_for_lane(&lane, &snapshot.remote_gates)
                    .ok_or_else(|| format!("unknown research lane `{lane}`"))?;
                if cli.json {
                    output::json(&packet)
                } else if target.is_some() {
                    output::write_text(target.as_deref(), &research::packet_markdown(&packet))
                } else {
                    output::research_packet(&packet);
                    Ok(())
                }
            }
            ResearchCommand::Synthesize { output: target } => {
                let synthesis = research::synthesis_readiness(&[]);
                if cli.json {
                    output::json(&synthesis)
                } else if target.is_some() {
                    output::write_text(target.as_deref(), &research::synthesis_markdown(&synthesis))
                } else {
                    output::research_synthesis(&synthesis);
                    Ok(())
                }
            }
        },
        Commands::Runs { command: _ } => {
            let snapshot = snapshot::build(ctx);
            if cli.json {
                output::json(&snapshot.runs)
            } else {
                output::runs(&snapshot);
                Ok(())
            }
        }
        Commands::Sc { command } => match command.unwrap_or(ScCommand::All) {
            ScCommand::All => {
                let report = sc::report();
                if cli.json {
                    output::json(&report)
                } else {
                    output::sc_report(&report);
                    Ok(())
                }
            }
            ScCommand::Status => {
                let status = sc::status();
                if cli.json {
                    output::json(&status)
                } else {
                    output::sc_status(&status);
                    Ok(())
                }
            }
            ScCommand::Sessions => {
                let sessions = sc::sessions();
                if cli.json {
                    output::json(&sessions)
                } else {
                    output::sc_sessions(&sessions);
                    Ok(())
                }
            }
            ScCommand::Providers => {
                let providers = sc::providers();
                if cli.json {
                    output::json(&providers)
                } else {
                    output::sc_providers(&providers);
                    Ok(())
                }
            }
            ScCommand::Worktree => {
                let worktree = sc::worktree();
                if cli.json {
                    output::json(&worktree)
                } else {
                    output::sc_worktree(&worktree);
                    Ok(())
                }
            }
        },
        Commands::Run { command } => match command {
            RunCommand::Verify {
                receipt: receipt_target,
                save,
            } => run_verify_command(ctx, cli.json, receipt_target, save),
        },
        Commands::Doctor => {
            let report = doctor(ctx);
            if cli.json {
                output::json(&report)
            } else {
                output::doctor(&report);
                Ok(())
            }
        }
        Commands::NativeAudit => {
            let report = audit::run(ctx);
            if cli.json {
                output::json(&report)
            } else {
                output::native_audit(&report);
                Ok(())
            }
        }
    }
}

fn run_chat_command(
    ctx: &Context,
    json: bool,
    cwd: Option<PathBuf>,
    receipt_target: Option<String>,
    save: bool,
    prompt: &str,
) -> RavenResult<()> {
    let turn = hermes::ask_with_options(ctx, prompt, hermes::HermesOptions { cwd })?;
    let chat_receipt = receipt::from_chat(&turn);

    if receipt_target.as_deref() == Some("-") {
        output::json(&chat_receipt)?;
    } else if json {
        output::json(&turn)?;
    } else {
        output::chat_turn(&turn);
    }

    if let Some(path) = receipt_target.as_deref().filter(|path| *path != "-") {
        let written = receipt::save_receipt(ctx, &chat_receipt, Some(path))?;
        output::line(&format!("RECEIPT: {}", written.display()));
    }

    if save {
        let written = receipt::save_receipt(ctx, &chat_receipt, None)?;
        output::line(&format!("SAVED: {}", written.display()));
    }

    Ok(())
}

pub fn dispatch_repl(ctx: &Context, input: &str) -> RavenResult<bool> {
    match input {
        "/help" => {
            println!("RAVEN_REPL_COMMANDS");
            println!("/help");
            println!("/status");
            println!("/packet");
            println!("/chat <prompt>");
            println!("/memory <query>");
            println!("/agents");
            println!("/gates");
            println!("/research [lane]");
            println!("/runs");
            println!("/sc [status|sessions|providers|worktree]");
            println!("/doctor");
            println!("/audit");
            println!("/quit");
        }
        "/status" => output::status(&snapshot::build(ctx)),
        "/packet" => output::packet(&snapshot::build(ctx)),
        "/agents" => output::agents(&snapshot::build(ctx)),
        "/gates" => output::gates(&snapshot::build(ctx)),
        "/research" => output::research_lanes(&research::list_lanes()),
        "/runs" => output::runs(&snapshot::build(ctx)),
        "/sc" => output::sc_report(&sc::report()),
        "/sc status" => output::sc_status(&sc::status()),
        "/sc sessions" => output::sc_sessions(&sc::sessions()),
        "/sc providers" => output::sc_providers(&sc::providers()),
        "/sc worktree" => output::sc_worktree(&sc::worktree()),
        "/doctor" => output::doctor(&doctor(ctx)),
        "/audit" => output::native_audit(&audit::run(ctx)),
        "/quit" | "/exit" => return Ok(false),
        _ if input.starts_with("/chat ") || input.starts_with("/hermes ") => {
            let prompt = input
                .trim_start_matches("/chat ")
                .trim_start_matches("/hermes ")
                .trim();
            output::chat_turn(&hermes::ask(ctx, prompt)?);
        }
        _ if input.starts_with("/memory ") => {
            let result = memory::search(ctx, input.trim_start_matches("/memory ").trim());
            output::memory_search(&result);
        }
        _ if input.starts_with("/research ") => {
            let lane = input.trim_start_matches("/research ").trim();
            let snapshot = snapshot::build(ctx);
            if let Some(packet) = research::packet_for_lane(lane, &snapshot.remote_gates) {
                output::research_packet(&packet);
            } else {
                output::line("VERDICT: FLAG");
                output::line(&format!("EVIDENCE: unknown research lane `{lane}`"));
                output::line("NEXT: /research");
            }
        }
        _ if input.starts_with('/') => {
            output::line("VERDICT: FLAG");
            output::line(&format!("EVIDENCE: unknown command `{input}`"));
            output::line("NEXT: /help");
        }
        _ => output::chat_turn(&hermes::ask(ctx, input)?),
    }
    Ok(true)
}

fn run_verify_command(
    ctx: &Context,
    json: bool,
    receipt_target: Option<String>,
    save: bool,
) -> RavenResult<()> {
    let result = verify::run_verify(ctx);
    let receipt = receipt::from_verify(&result);

    if json || receipt_target.as_deref() == Some("-") {
        output::json(&receipt)?;
    } else {
        output::verify_human(&receipt);
    }

    if let Some(path) = receipt_target.as_deref().filter(|path| *path != "-") {
        let written = receipt::save_receipt(ctx, &receipt, Some(path))?;
        output::line(&format!("RECEIPT_WRITTEN: {}", written.display()));
    }
    if save {
        let written = receipt::save_receipt(ctx, &receipt, None)?;
        output::line(&format!("RECEIPT_SAVED: {}", written.display()));
    }

    if result.exit_code == 0 {
        Ok(())
    } else {
        Err(format!("local verifier exited {}", result.exit_code).into())
    }
}

fn doctor(ctx: &Context) -> DoctorReport {
    let mut checks = Vec::new();
    for (program, args) in [
        ("rustc", vec!["--version"]),
        ("cargo", vec!["--version"]),
        ("just", vec!["--version"]),
        ("bun", vec!["--version"]),
        ("node", vec!["--version"]),
        ("python3", vec!["--version"]),
        ("multica", vec!["--version"]),
    ] {
        checks.push(command_check(program, &args));
    }

    for required in crate::constants::REQUIRED_DOCS {
        let path = ctx.root.join(required);
        checks.push(DoctorCheck {
            name: format!("file {required}"),
            verdict: if path.exists() {
                Verdict::Pass
            } else {
                Verdict::Block
            },
            evidence: if path.exists() {
                "present".to_string()
            } else {
                "missing".to_string()
            },
        });
    }

    checks.push(deepseek_auth_check(ctx));

    let gitignore = ctx.root.join(".gitignore");
    let ignored = std::fs::read_to_string(&gitignore)
        .map(|text| text.contains("raven/.local-runs/"))
        .unwrap_or(false);
    checks.push(DoctorCheck {
        name: "gitignore raven/.local-runs".to_string(),
        verdict: if ignored {
            Verdict::Pass
        } else {
            Verdict::Block
        },
        evidence: if ignored {
            "saved receipts are gitignored".to_string()
        } else {
            "saved receipts are not gitignored".to_string()
        },
    });

    let memory = memory::health(ctx);
    checks.push(DoctorCheck {
        name: "memory bridge health".to_string(),
        verdict: memory.verdict,
        evidence: memory.evidence,
    });

    let verdict = if checks.iter().any(|check| check.verdict == Verdict::Block) {
        Verdict::Block
    } else if checks.iter().any(|check| check.verdict == Verdict::Flag) {
        Verdict::Flag
    } else {
        Verdict::Pass
    };

    DoctorReport {
        verdict,
        checks,
        next: "run raven run verify, raven gates, and raven native-audit before closeout."
            .to_string(),
    }
}

fn deepseek_auth_check(ctx: &Context) -> DoctorCheck {
    let script = ctx.root.join("scripts/deepseek-auth-preflight.sh");
    let env_file = ctx.root.join("deploy/nixos/evercore.env.example");
    match Command::new(&script).arg("--env").arg(&env_file).output() {
        Ok(output) if output.status.success() => {
            let text = if output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stderr)
            } else {
                String::from_utf8_lossy(&output.stdout)
            };
            DoctorCheck {
                name: "deepseek auth preflight".to_string(),
                verdict: Verdict::Pass,
                evidence: crate::sanitizer::sanitize_text(&crate::util::one_line(&text)),
            }
        }
        Ok(output) => DoctorCheck {
            name: "deepseek auth preflight".to_string(),
            verdict: Verdict::Block,
            evidence: format!("exited {}", output.status),
        },
        Err(err) => DoctorCheck {
            name: "deepseek auth preflight".to_string(),
            verdict: Verdict::Block,
            evidence: err.to_string(),
        },
    }
}

fn command_check(program: &str, args: &[&str]) -> DoctorCheck {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => {
            let text = if output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stderr)
            } else {
                String::from_utf8_lossy(&output.stdout)
            };
            DoctorCheck {
                name: program.to_string(),
                verdict: Verdict::Pass,
                evidence: crate::sanitizer::sanitize_text(&crate::util::one_line(&text)),
            }
        }
        Ok(output) => DoctorCheck {
            name: program.to_string(),
            verdict: if program == "multica" {
                Verdict::Flag
            } else {
                Verdict::Block
            },
            evidence: format!("exited {}", output.status),
        },
        Err(err) => DoctorCheck {
            name: program.to_string(),
            verdict: if program == "multica" {
                Verdict::Flag
            } else {
                Verdict::Block
            },
            evidence: err.to_string(),
        },
    }
}

#[allow(dead_code)]
fn relative_path_exists(root: &Path, relative: &str) -> bool {
    root.join(relative).exists()
}
