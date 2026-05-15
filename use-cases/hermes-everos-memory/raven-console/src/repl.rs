use crate::commands;
use crate::context::Context;
use crate::output;
use crate::RavenResult;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io::{self, BufRead, IsTerminal};

pub fn run(ctx: &Context) -> RavenResult<()> {
    println!("Raven REPL. Type /help or /quit.");
    if !io::stdin().is_terminal() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let input = line?;
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            if !commands::dispatch_repl(ctx, input)? {
                break;
            }
        }
        return Ok(());
    }

    let mut editor = DefaultEditor::new()?;
    loop {
        match editor.readline("raven> ") {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = editor.add_history_entry(input);
                if !commands::dispatch_repl(ctx, input)? {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("INTERRUPT");
                break;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err.into()),
        }
        output::flush_stdout()?;
    }

    Ok(())
}
