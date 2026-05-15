use crate::adapters::packet::read_packet;
use crate::constants::FIXTURE_PATH;
use crate::model::RunPacket;
use crate::RavenResult;
use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Context {
    pub root: PathBuf,
    pub packet: RunPacket,
}

impl Context {
    pub fn load() -> RavenResult<Self> {
        let root = find_case_root()?;
        let packet = read_packet(&root.join(FIXTURE_PATH))?;
        Ok(Self { root, packet })
    }
}

fn find_case_root() -> RavenResult<PathBuf> {
    let cwd = env::current_dir()?;
    for candidate in cwd.ancestors() {
        let direct = candidate.join("COMPLETION_AUDIT.md");
        let fixture = candidate.join(FIXTURE_PATH);
        if direct.exists() && fixture.exists() {
            return Ok(candidate.to_path_buf());
        }

        let nested = candidate.join("use-cases/hermes-everos-memory");
        if nested.join("COMPLETION_AUDIT.md").exists() && nested.join(FIXTURE_PATH).exists() {
            return Ok(nested);
        }
    }

    Err("could not find use-cases/hermes-everos-memory root".into())
}
