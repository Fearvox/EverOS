use crate::adapters::hermes;
use crate::context::Context;
use crate::model::{HermesChatTranscriptLine, HermesChatTurn, RavenSnapshot, Verdict};
use crate::sanitizer::sanitize_text;
use crate::snapshot;
use crate::RavenResult;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{CrosstermBackend, TestBackend};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::collections::VecDeque;
use std::env;
use std::io::{self, IsTerminal};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Panel {
    Status,
    Packet,
    Chat,
    Memory,
    Agents,
    Gates,
    Runs,
    Sc,
    Doctor,
    NativeAudit,
    Help,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMode {
    Normal,
    Palette,
    Search,
    Chat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TuiAction {
    Continue,
    Quit,
    Refresh,
    SendChat(String),
}

struct TuiState {
    panel: Panel,
    mode: InputMode,
    input: String,
    evidence: String,
    chat: VecDeque<ChatLine>,
    chat_inflight: bool,
}

struct ChatLine {
    role: &'static str,
    text: String,
    verdict: Option<Verdict>,
}

enum BackgroundEvent {
    Snapshot(Box<RavenSnapshot>),
    Chat(HermesChatTurn),
}

const SURFACE_TITLE: &str = "RAVEN // DOOMSDAY-MAXXED-MOGGED";
const CHAT_HISTORY_LIMIT: usize = 24;

impl Default for TuiState {
    fn default() -> Self {
        Self {
            panel: Panel::Status,
            mode: InputMode::Normal,
            input: String::new(),
            evidence: "Remote gates stay red until live evidence proves every hard gate."
                .to_string(),
            chat: VecDeque::new(),
            chat_inflight: false,
        }
    }
}

pub fn run(ctx: &Context) -> RavenResult<()> {
    if env::var("RAVEN_TUI_ONCE").is_ok() || !io::stdout().is_terminal() {
        let snapshot = snapshot::build_tui_boot(ctx);
        let state = TuiState::default();
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|frame| render(frame, &snapshot, &state))?;
        print!("{}", buffer_to_string(terminal.backend().buffer()));
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_loop(ctx, &mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop(
    ctx: &Context,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> RavenResult<()> {
    let mut state = TuiState::default();
    let mut snapshot = snapshot::build_tui_boot(ctx);
    let (tx, rx) = mpsc::channel();
    let mut refresh_inflight = start_live_refresh(ctx.clone(), tx.clone());
    state.evidence =
        "Fast boot snapshot is on screen. Live Multica/memory refresh is running.".to_string();

    loop {
        while let Some(event) = receive_background_event(&rx) {
            match event {
                BackgroundEvent::Snapshot(next) => {
                    snapshot = *next;
                    refresh_inflight = false;
                    state.evidence = "Live refresh complete. Press u to refresh again.".to_string();
                }
                BackgroundEvent::Chat(turn) => {
                    state.chat_inflight = false;
                    state.evidence = turn.evidence.clone();
                    push_chat_line(
                        &mut state,
                        ChatLine {
                            role: "hermes",
                            text: turn.response,
                            verdict: Some(turn.verdict),
                        },
                    );
                }
            }
        }

        terminal.draw(|frame| render(frame, &snapshot, &state))?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match handle_key(key, &mut state) {
                    TuiAction::Quit => break,
                    TuiAction::Refresh => {
                        if refresh_inflight {
                            state.evidence = "Live refresh already running.".to_string();
                        } else {
                            refresh_inflight = start_live_refresh(ctx.clone(), tx.clone());
                            state.evidence = "Live Multica/memory refresh started.".to_string();
                        }
                    }
                    TuiAction::SendChat(prompt) => {
                        if state.chat_inflight {
                            state.evidence = "Hermes turn already running.".to_string();
                        } else {
                            state.chat_inflight = start_chat_turn(ctx.clone(), prompt, tx.clone());
                            state.evidence = "Hermes turn started in background.".to_string();
                        }
                    }
                    TuiAction::Continue => {}
                },
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
    Ok(())
}

fn start_live_refresh(ctx: Context, tx: Sender<BackgroundEvent>) -> bool {
    thread::spawn(move || {
        let snapshot = snapshot::build(&ctx);
        let _ = tx.send(BackgroundEvent::Snapshot(Box::new(snapshot)));
    });
    true
}

fn start_chat_turn(ctx: Context, prompt: String, tx: Sender<BackgroundEvent>) -> bool {
    thread::spawn(move || {
        let turn = hermes::ask(&ctx, &prompt).unwrap_or_else(|err| HermesChatTurn {
            prompt: sanitize_text(&prompt),
            command: vec![
                "hermes".to_string(),
                "-z".to_string(),
                "[raven-prompt]".to_string(),
            ],
            workspace: "case-root".to_string(),
            runtime: "unknown".to_string(),
            verdict: Verdict::Flag,
            exit_code: 1,
            duration_ms: 0,
            response: "Hermes turn failed before producing output.".to_string(),
            evidence: sanitize_text(&format!("Hermes adapter error: {err}")),
            transcript: vec![HermesChatTranscriptLine {
                role: "operator".to_string(),
                content: sanitize_text(&prompt),
            }],
        });
        let _ = tx.send(BackgroundEvent::Chat(turn));
    });
    true
}

fn push_chat_line(state: &mut TuiState, line: ChatLine) {
    if state.chat.len() == CHAT_HISTORY_LIMIT {
        state.chat.pop_front();
    }
    state.chat.push_back(line);
}

fn receive_background_event(rx: &Receiver<BackgroundEvent>) -> Option<BackgroundEvent> {
    match rx.try_recv() {
        Ok(event) => Some(event),
        Err(mpsc::TryRecvError::Empty) | Err(mpsc::TryRecvError::Disconnected) => None,
    }
}

fn handle_key(key: KeyEvent, state: &mut TuiState) -> TuiAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return TuiAction::Quit;
    }

    match state.mode {
        InputMode::Normal => handle_normal_key(key, state),
        InputMode::Palette | InputMode::Search | InputMode::Chat => handle_input_key(key, state),
    }
}

fn handle_normal_key(key: KeyEvent, state: &mut TuiState) -> TuiAction {
    match key.code {
        KeyCode::Char('q') => return TuiAction::Quit,
        KeyCode::Char('u') => return TuiAction::Refresh,
        KeyCode::Char('?') => state.panel = Panel::Help,
        KeyCode::Char('h') | KeyCode::Char('c') => state.panel = Panel::Chat,
        KeyCode::Char('i') | KeyCode::Enter if state.panel == Panel::Chat => {
            state.mode = InputMode::Chat;
            state.input.clear();
            state.evidence = "Hermes input mode. Enter sends; Esc cancels.".to_string();
        }
        KeyCode::Char(':') => {
            state.mode = InputMode::Palette;
            state.input.clear();
            state.evidence =
                "Palette mode. Type a panel name, Enter to apply, Esc to cancel.".to_string();
        }
        KeyCode::Char('/') => {
            state.mode = InputMode::Search;
            state.input.clear();
            state.panel = Panel::Memory;
            state.evidence =
                "Search mode. Type a memory query, Enter to keep it in the evidence drawer."
                    .to_string();
        }
        KeyCode::Char('s') => state.panel = Panel::Status,
        KeyCode::Char('p') => state.panel = Panel::Packet,
        KeyCode::Char('m') => state.panel = Panel::Memory,
        KeyCode::Char('a') => state.panel = Panel::Agents,
        KeyCode::Char('g') => state.panel = Panel::Gates,
        KeyCode::Char('r') => state.panel = Panel::Runs,
        KeyCode::Char('o') => state.panel = Panel::Sc,
        KeyCode::Char('d') => state.panel = Panel::Doctor,
        KeyCode::Char('n') => state.panel = Panel::NativeAudit,
        KeyCode::Esc => state.panel = Panel::Status,
        _ => {}
    }
    TuiAction::Continue
}

fn handle_input_key(key: KeyEvent, state: &mut TuiState) -> TuiAction {
    match key.code {
        KeyCode::Esc => {
            state.mode = InputMode::Normal;
            state.input.clear();
            state.evidence = "Input cancelled.".to_string();
        }
        KeyCode::Enter => {
            if state.mode == InputMode::Palette {
                apply_palette(&state.input.clone(), state);
            } else if state.mode == InputMode::Search {
                state.evidence = format!(
                    "Memory search query staged: `{}`. Use `raven memory search {}` for full bridge output.",
                    state.input, state.input
                );
            } else {
                let prompt = state.input.trim().to_string();
                if prompt.is_empty() {
                    state.evidence = "Hermes prompt is empty.".to_string();
                } else if state.chat_inflight {
                    state.evidence = "Hermes turn already running.".to_string();
                } else {
                    state.panel = Panel::Chat;
                    push_chat_line(
                        state,
                        ChatLine {
                            role: "you",
                            text: sanitize_text(&prompt),
                            verdict: None,
                        },
                    );
                    push_chat_line(
                        state,
                        ChatLine {
                            role: "system",
                            text: "queued Hermes turn; UI remains live".to_string(),
                            verdict: Some(Verdict::Flag),
                        },
                    );
                    state.mode = InputMode::Normal;
                    state.input.clear();
                    return TuiAction::SendChat(prompt);
                }
            }
            state.mode = InputMode::Normal;
            state.input.clear();
        }
        KeyCode::Backspace => {
            state.input.pop();
        }
        KeyCode::Char(ch) => state.input.push(ch),
        _ => {}
    }
    TuiAction::Continue
}

fn apply_palette(input: &str, state: &mut TuiState) {
    match input.trim().to_ascii_lowercase().as_str() {
        "status" | "s" => state.panel = Panel::Status,
        "packet" | "p" => state.panel = Panel::Packet,
        "chat" | "hermes" | "h" | "c" => state.panel = Panel::Chat,
        "memory" | "m" => state.panel = Panel::Memory,
        "agents" | "a" => state.panel = Panel::Agents,
        "gates" | "g" => state.panel = Panel::Gates,
        "runs" | "r" => state.panel = Panel::Runs,
        "sc" | "superconductor" | "conductor" | "o" => state.panel = Panel::Sc,
        "doctor" | "d" => state.panel = Panel::Doctor,
        "audit" | "native" | "n" => state.panel = Panel::NativeAudit,
        "help" | "?" => state.panel = Panel::Help,
        other => state.evidence = format!("Unknown palette command `{other}`."),
    }
}

fn render(frame: &mut Frame<'_>, snapshot: &RavenSnapshot, state: &TuiState) {
    let root = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(12),
            Constraint::Length(4),
        ])
        .split(root);

    render_status(frame, vertical[0], snapshot);
    render_body(frame, vertical[1], snapshot, state);
    render_input(frame, vertical[2], state);
}

fn render_status(frame: &mut Frame<'_>, area: Rect, snapshot: &RavenSnapshot) {
    let lines = vec![
        Line::from(vec![
            Span::styled(
                "CONTROL ROOM",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " / local-first memory OS / ",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                "remote truth stays red until proven",
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            chip("OVERALL", snapshot.verdict.to_string()),
            Span::raw("  "),
            chip("LOCAL", snapshot.packet.verdict.to_string()),
            Span::raw("  "),
            chip("MEMORY", snapshot.memory.verdict.to_string()),
            Span::raw("  "),
            chip("DAS-2666", gate_verdict(snapshot, "DAS-2666")),
            Span::raw("  "),
            chip("DAS-2669", gate_verdict(snapshot, "DAS-2669")),
        ]),
        Line::from(vec![
            Span::styled("WATCH ", Style::default().fg(Color::DarkGray)),
            Span::styled("2670", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("2671", Style::default().fg(Color::Cyan)),
            Span::raw(" / "),
            Span::styled("2672", Style::default().fg(Color::Cyan)),
            Span::styled(
                "     adapters isolated: DAS-2675 cannot green DAS-2666",
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(shell_block(SURFACE_TITLE, Color::Cyan)),
        area,
    );
}

fn render_body(frame: &mut Frame<'_>, area: Rect, snapshot: &RavenSnapshot, state: &TuiState) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(31),
            Constraint::Min(41),
            Constraint::Length(48),
        ])
        .split(area);

    render_rail(frame, horizontal[0], state.panel);
    render_panel(frame, horizontal[1], snapshot, state);
    render_evidence(frame, horizontal[2], snapshot, state);
}

fn render_rail(frame: &mut Frame<'_>, area: Rect, active: Panel) {
    let items = [
        ("s", "Status", "truth stack", Panel::Status),
        ("p", "Packet", "owner view", Panel::Packet),
        ("h", "Hermes Chat", "dialogue", Panel::Chat),
        ("m", "Memory", "bridge health", Panel::Memory),
        ("a", "Agents", "watch lanes", Panel::Agents),
        ("g", "Gates", "hard stops", Panel::Gates),
        ("r", "Runs", "receipts", Panel::Runs),
        ("o", "SC", "conductor", Panel::Sc),
        ("d", "Doctor", "toolchain", Panel::Doctor),
        ("n", "Native Audit", "UX safety", Panel::NativeAudit),
        ("?", "Help", "keys", Panel::Help),
    ]
    .into_iter()
    .map(|(key, label, detail, panel)| {
        let active_style = if panel == active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let marker = if panel == active { ">" } else { " " };
        ListItem::new(Line::from(vec![
            Span::styled(marker, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(format!("[{key}] "), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{label:<13}"), active_style),
            Span::raw(" "),
            Span::styled(detail, Style::default().fg(Color::DarkGray)),
        ]))
    })
    .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(shell_block("COMMAND RAIL", Color::DarkGray)),
        area,
    );
}

fn render_panel(frame: &mut Frame<'_>, area: Rect, snapshot: &RavenSnapshot, state: &TuiState) {
    let (title, lines) = match state.panel {
        Panel::Status => ("Status", status_lines(snapshot)),
        Panel::Packet => ("Packet", packet_lines(snapshot)),
        Panel::Chat => ("Hermes Chat", chat_lines(state)),
        Panel::Memory => ("Memory", memory_lines(snapshot)),
        Panel::Agents => ("Agents", agent_lines(snapshot)),
        Panel::Gates => ("Gates", gate_lines(snapshot)),
        Panel::Runs => ("Runs", run_lines(snapshot)),
        Panel::Sc => ("Superconductor", sc_lines(snapshot)),
        Panel::Doctor => ("Doctor", doctor_lines()),
        Panel::NativeAudit => ("Native Audit", native_lines()),
        Panel::Help => ("Help", help_lines()),
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(shell_block(title, panel_color(state.panel)))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_evidence(frame: &mut Frame<'_>, area: Rect, snapshot: &RavenSnapshot, state: &TuiState) {
    let mut lines = vec![
        section("ACTIVE EVIDENCE"),
        Line::from(vec![Span::styled(
            state.evidence.clone(),
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
        section("REMOTE HARD GATES"),
    ];
    for gate in &snapshot.remote_gates {
        lines.push(Line::from(vec![
            verdict_span(gate.verdict.to_string()),
            Span::raw(" "),
            Span::styled(format!("{:<8}", gate.id), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(gate.evidence.clone(), Style::default().fg(Color::Gray)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(section("RISK REGISTER"));
    for risk in &snapshot.risks {
        lines.push(Line::from(vec![
            Span::styled("- ", Style::default().fg(Color::Yellow)),
            Span::styled(risk.clone(), Style::default().fg(Color::Gray)),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(shell_block("EVIDENCE DRAWER", Color::Yellow))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_input(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let (title, prompt, color) = match state.mode {
        InputMode::Normal => (
            "INPUT // NORMAL",
            "keys: h chat | i input | u refresh | ? help | : palette | / memory | s/p/m/a/g/r/o/d/n panels | q quit"
                .to_string(),
            Color::DarkGray,
        ),
        InputMode::Palette => (
            "INPUT // PALETTE",
            format!("route > {}", state.input),
            Color::Cyan,
        ),
        InputMode::Search => (
            "INPUT // MEMORY",
            format!("query > {}", state.input),
            Color::Green,
        ),
        InputMode::Chat => (
            "INPUT // HERMES",
            format!("hermes > {}", state.input),
            Color::Magenta,
        ),
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("RAVEN ", Style::default().fg(Color::Cyan)),
            Span::styled(prompt, Style::default().fg(Color::Gray)),
        ]))
        .block(shell_block(title, color)),
        area,
    );
}

fn status_lines(snapshot: &RavenSnapshot) -> Vec<Line<'static>> {
    let mut lines = vec![
        section("VERDICT STACK"),
        Line::from(vec![
            chip("overall", snapshot.verdict.to_string()),
            Span::raw("  "),
            chip("packet", snapshot.packet.verdict.to_string()),
            Span::raw("  "),
            chip("memory", snapshot.memory.verdict.to_string()),
        ]),
        Line::from(""),
        section("FIRST WATCH"),
    ];
    for id in ["DAS-2670", "DAS-2671", "DAS-2672"] {
        if let Some(issue) = snapshot
            .watchlist_issues
            .iter()
            .find(|issue| issue.id == id)
        {
            lines.push(issue_line(
                issue.id.clone(),
                issue.status.clone(),
                issue.title.clone(),
            ));
        }
    }
    lines.push(Line::from(""));
    lines.push(section("REMOTE STOPS"));
    for issue in &snapshot.watchlist_issues {
        if issue.id == "DAS-2666" || issue.id == "DAS-2669" || issue.id == "DAS-2675" {
            lines.push(issue_line(
                issue.id.clone(),
                issue.status.clone(),
                issue.title.clone(),
            ));
        }
    }
    lines
}

fn packet_lines(snapshot: &RavenSnapshot) -> Vec<Line<'static>> {
    let mut lines = vec![
        section("OWNER PACKET"),
        kv("id", &snapshot.packet.id),
        kv("title", &snapshot.packet.title),
        kv("status", &snapshot.packet.status),
        kv("owners", &snapshot.packet.owners.join(", ")),
        Line::from(""),
        section("SOURCE DOCS"),
    ];
    for doc in &snapshot.packet.docs {
        lines.push(Line::from(vec![
            verdict_span(doc.verdict.to_string()),
            Span::raw(" "),
            Span::styled(
                format!("{:<26}", doc.path),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" "),
            Span::styled(doc.evidence.clone(), Style::default().fg(Color::Gray)),
        ]));
    }
    lines
}

fn memory_lines(snapshot: &RavenSnapshot) -> Vec<Line<'static>> {
    vec![
        section("MEMORY BRIDGE"),
        Line::from(vec![
            chip("health", snapshot.memory.verdict.to_string()),
            Span::raw("  "),
            chip("status", snapshot.memory.status.clone()),
        ]),
        kv("evidence", &snapshot.memory.evidence),
        Line::from(""),
        Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Cyan)),
            Span::styled(
                " opens staged memory-search input; u refreshes live bridge/watch data.",
                Style::default().fg(Color::Gray),
            ),
        ]),
    ]
}

fn chat_lines(state: &TuiState) -> Vec<Line<'static>> {
    let mut lines = vec![
        section("HERMES REPL WINDOW"),
        Line::from(vec![
            chip(
                "state",
                if state.chat_inflight {
                    "RUNNING".to_string()
                } else {
                    "READY".to_string()
                },
            ),
            Span::raw("  "),
            Span::styled(
                "h opens this panel; i starts prompt input; Enter sends.",
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(""),
    ];

    if state.chat.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("transcript", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(
                "empty; this TUI window shares the same Hermes adapter as `raven chat send` and `raven repl`.",
                Style::default().fg(Color::Gray),
            ),
        ]));
        return lines;
    }

    for line in &state.chat {
        let label = match line.verdict {
            Some(verdict) => format!("{} [{}]", line.role, verdict),
            None => line.role.to_string(),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{label:<16}"),
                Style::default().fg(role_color(line.role)),
            ),
            Span::styled(line.text.clone(), Style::default().fg(Color::Gray)),
        ]));
    }

    lines
}

fn agent_lines(snapshot: &RavenSnapshot) -> Vec<Line<'static>> {
    let mut lines = vec![section("LANE CONTROL")];
    for agent in &snapshot.agents {
        lines.push(Line::from(vec![
            verdict_span(agent.verdict.to_string()),
            Span::raw(" "),
            Span::styled(
                format!("{:<22}", agent.name),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("{:<10}", agent.status),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!("({})", agent.issue_id),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }
    lines
}

fn gate_lines(snapshot: &RavenSnapshot) -> Vec<Line<'static>> {
    let mut lines = vec![section("REMOTE HARD GATES")];
    for gate in &snapshot.remote_gates {
        lines.push(Line::from(vec![
            verdict_span(gate.verdict.to_string()),
            Span::raw(" "),
            Span::styled(format!("{:<8}", gate.id), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                format!("blocks={} hard={} ", gate.blocks_completion, gate.hard_gate),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(gate.evidence.clone(), Style::default().fg(Color::Gray)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(section("LOCAL PACKET GATES"));
    for gate in &snapshot.local_gates {
        lines.push(Line::from(vec![
            verdict_span(gate.verdict.to_string()),
            Span::raw(" "),
            Span::styled(
                format!("{:<24}", gate.id),
                Style::default().fg(Color::White),
            ),
            Span::styled(gate.command.clone(), Style::default().fg(Color::Gray)),
        ]));
    }
    lines
}

fn run_lines(snapshot: &RavenSnapshot) -> Vec<Line<'static>> {
    let mut lines = vec![section("RUN RECEIPTS")];
    for run in &snapshot.runs {
        lines.push(Line::from(vec![
            verdict_span(run.verdict.to_string()),
            Span::raw(" "),
            Span::styled(format!("{:<28}", run.id), Style::default().fg(Color::White)),
            Span::styled(
                format!("{} ", run.source),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(run.command.clone(), Style::default().fg(Color::Gray)),
        ]));
    }
    lines
}

fn sc_lines(snapshot: &RavenSnapshot) -> Vec<Line<'static>> {
    let api_version = snapshot
        .sc
        .status
        .api_version
        .map(|version| version.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = snapshot
        .sc
        .worktree
        .dirty
        .map(|dirty| dirty.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut lines = vec![
        section("SUPERCONDUCTOR"),
        kv("verdict", &snapshot.sc.verdict.to_string()),
        kv("api", &api_version),
        kv("app", &snapshot.sc.status.app_version),
        kv("status", &snapshot.sc.status.evidence),
        Line::from(""),
        section("WORKTREE"),
        kv("branch", &snapshot.sc.worktree.branch),
        kv("target", &snapshot.sc.worktree.target_branch),
        kv("dirty", &dirty),
        kv("evidence", &snapshot.sc.worktree.evidence),
        Line::from(""),
        section("SESSIONS"),
    ];

    if snapshot.sc.sessions.is_empty() {
        lines.push(Line::from("none or unavailable"));
    } else {
        for session in snapshot.sc.sessions.iter().take(6) {
            lines.push(Line::from(vec![
                verdict_span(if session.closed { "FLAG" } else { "PASS" }.to_string()),
                Span::raw(" "),
                Span::styled(
                    format!("{:<7}", session.provider_key),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" "),
                Span::styled(session.model.clone(), Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(
                    if session.active_turn {
                        "active"
                    } else {
                        "idle"
                    },
                    Style::default().fg(if session.active_turn {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::raw(" "),
                Span::styled(session.title.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(section("PROVIDERS"));
    for provider in snapshot.sc.providers.iter().take(5) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<7}", provider.provider_key),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!(
                    " enabled={} models={}",
                    provider.enabled, provider.model_count
                ),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }

    lines
}

fn doctor_lines() -> Vec<Line<'static>> {
    vec![
        section("DOCTOR"),
        Line::from(vec![
            Span::styled("Use ", Style::default().fg(Color::Gray)),
            Span::styled("raven doctor", Style::default().fg(Color::Cyan)),
            Span::styled(
                " for dependency/file checks. This pane is intentionally non-mutating.",
                Style::default().fg(Color::Gray),
            ),
        ]),
    ]
}

fn native_lines() -> Vec<Line<'static>> {
    vec![
        section("NATIVE AUDIT"),
        Line::from(vec![
            Span::styled("Use ", Style::default().fg(Color::Gray)),
            Span::styled("raven native-audit", Style::default().fg(Color::Cyan)),
            Span::styled(" for UX/safety gates.", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            verdict_span("BLOCK".to_string()),
            Span::styled(
                " hard failures block PASS.",
                Style::default().fg(Color::Gray),
            ),
        ]),
    ]
}

fn help_lines() -> Vec<Line<'static>> {
    vec![
        section("KEYMAP"),
        kv("?", "help"),
        kv(":", "palette"),
        kv("/", "memory/search"),
        kv("h/c", "Hermes chat panel"),
        kv("i", "prompt input when Hermes panel is active"),
        kv("u", "refresh live Multica + memory data"),
        kv(
            "panels",
            "s status | p packet | h chat | m memory | a agents",
        ),
        kv("panels", "g gates | r runs | d doctor | n native audit"),
        kv("panels", "o superconductor"),
        kv("exit", "Esc cancel | Ctrl-C/q quit"),
    ]
}

fn gate_verdict(snapshot: &RavenSnapshot, id: &str) -> String {
    snapshot
        .remote_gates
        .iter()
        .find(|gate| gate.id == id)
        .map(|gate| gate.verdict.to_string())
        .unwrap_or_else(|| "FLAG".to_string())
}

fn buffer_to_string(buffer: &Buffer) -> String {
    let width = buffer.area.width as usize;
    let mut output = String::new();
    for row in buffer.content.chunks(width) {
        for cell in row {
            output.push_str(cell.symbol());
        }
        output.push('\n');
    }
    output
}

fn shell_block(title: &'static str, accent: Color) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::QuadrantOutside)
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(Color::Black))
}

fn panel_color(panel: Panel) -> Color {
    match panel {
        Panel::Status => Color::Cyan,
        Panel::Packet => Color::Magenta,
        Panel::Chat => Color::Magenta,
        Panel::Memory => Color::Green,
        Panel::Agents => Color::Blue,
        Panel::Gates => Color::Red,
        Panel::Runs => Color::Yellow,
        Panel::Sc => Color::LightBlue,
        Panel::Doctor => Color::Gray,
        Panel::NativeAudit => Color::LightCyan,
        Panel::Help => Color::White,
    }
}

fn role_color(role: &str) -> Color {
    match role {
        "you" => Color::Cyan,
        "hermes" => Color::Magenta,
        "system" => Color::Yellow,
        _ => Color::Gray,
    }
}

fn verdict_span(value: String) -> Span<'static> {
    Span::styled(
        format!("[{value}]"),
        Style::default()
            .fg(verdict_color(&value))
            .add_modifier(Modifier::BOLD),
    )
}

fn chip(label: &'static str, value: String) -> Span<'static> {
    Span::styled(
        format!("{label} [{value}]"),
        Style::default()
            .fg(verdict_color(&value))
            .add_modifier(Modifier::BOLD),
    )
}

fn verdict_color(value: &str) -> Color {
    match value.to_ascii_uppercase().as_str() {
        "PASS" | "HEALTHY" => Color::Green,
        "BLOCK" | "BLOCKED" => Color::Red,
        "FLAG" | "IN_REVIEW" => Color::Yellow,
        _ => Color::Gray,
    }
}

fn section(label: &'static str) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("-- {label}"),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )])
}

fn kv(label: &'static str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<10}"), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(Color::Gray)),
    ])
}

fn issue_line(id: String, status: String, title: String) -> Line<'static> {
    let status_display = compact_status(&status);
    Line::from(vec![
        Span::styled(format!("{id:<8}"), Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(
            format!("{status_display:<12}"),
            Style::default().fg(verdict_color(&status)),
        ),
        Span::raw(" "),
        Span::styled(title, Style::default().fg(Color::Gray)),
    ])
}

fn compact_status(status: &str) -> String {
    let mut text = status.to_string();
    if text.len() > 12 {
        text.truncate(12);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_history_is_bounded_fifo() {
        let mut state = TuiState::default();

        for index in 0..30 {
            push_chat_line(
                &mut state,
                ChatLine {
                    role: "you",
                    text: format!("turn-{index}"),
                    verdict: None,
                },
            );
        }

        assert_eq!(state.chat.len(), CHAT_HISTORY_LIMIT);
        assert_eq!(
            state.chat.front().map(|line| line.text.as_str()),
            Some("turn-6")
        );
        assert_eq!(
            state.chat.back().map(|line| line.text.as_str()),
            Some("turn-29")
        );
    }
}
