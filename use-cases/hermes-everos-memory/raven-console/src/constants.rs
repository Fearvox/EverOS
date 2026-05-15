pub const FIXTURE_PATH: &str = "raven/fixtures/doomsday-run.json";
pub const RUNS_DIR: &str = "raven/.local-runs";

pub const ISSUE_REMOTE_DEPLOY: &str = "DAS-2666";
pub const ISSUE_AUTH_BLOCKER: &str = "DAS-2669";
pub const ISSUE_CONTROL_ROOM: &str = "DAS-2670";
pub const ISSUE_LOCAL_VERIFIER: &str = "DAS-2671";
pub const ISSUE_MEMORY_WATCH: &str = "DAS-2672";
pub const ISSUE_ADAPTER_REPAIR: &str = "DAS-2675";

pub const WATCHLIST_ISSUES: &[&str] = &[
    ISSUE_REMOTE_DEPLOY,
    ISSUE_AUTH_BLOCKER,
    ISSUE_CONTROL_ROOM,
    ISSUE_LOCAL_VERIFIER,
    ISSUE_MEMORY_WATCH,
    ISSUE_ADAPTER_REPAIR,
];

pub const REQUIRED_DOCS: &[&str] = &[
    "COMPLETION_AUDIT.md",
    "OWNER_PACKET.md",
    "SUPERVISOR_DISPATCH.md",
    FIXTURE_PATH,
    "raven/NATIVE_FEEL_AUDIT.md",
    "bin/raven-run.mjs",
    "bin/everos-memory.mjs",
];
