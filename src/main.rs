use crate::{
    branch::{BranchItem, checkout_branch, get_branches},
    term::{Term, Vec2},
};
use clap::Parser;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{Attribute, Color},
};
use git2::Repository;
use std::{path::Path, process::exit};

mod branch;
mod term;

const EVENT_POLL_TIMEOUT_MS: u64 = 10_000;
const PADDING: usize = 2;

/// Git tui tool
#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to repository
    #[arg(short, long, default_value = ".")]
    directory: String,

    /// Latest commit summary max length
    #[arg(short, long, default_value_t = 72)]
    summary_length: usize,

    /// Render debug info
    #[arg(short = 'D', long, action = clap::ArgAction::SetTrue)]
    debug: bool,
}

struct State {
    renders: usize,
    repo: Repository,
    branches: Vec<BranchItem>,
    selected_row: usize,
    search_string: String,
}

impl State {
    fn new(repo: Repository) -> Self {
        Self {
            renders: 0,
            repo,
            branches: Vec::new(),
            selected_row: 0,
            search_string: String::new(),
        }
    }
}

fn main() {
    let args = Args::parse();
    let mut do_run = true;
    let mut do_render = true;
    let mut do_search = false;

    let directory = Path::new(&args.directory);
    let repo = match Repository::open(directory) {
        Ok(repo) => repo,
        Err(err) => {
            eprintln!("fatal: {}", err.message());
            exit(1);
        }
    };
    let mut state = State::new(repo);

    let mut term = Term::new();
    term.clear_all();
    while do_run {
        if do_render {
            render_branches(&mut term, &mut state, &args);
            if do_search || !state.search_string.is_empty() {
                let max_y = (Term::size().y) as usize - PADDING;
                term.write_text(
                    Vec2::from((PADDING, max_y)),
                    format!("/ {}", state.search_string),
                );
            }

            if args.debug {
                render_debug_info(&mut term, &mut state, &args);
            }
            do_render = false;
        }
        if let Some(event) = term.read_event(EVENT_POLL_TIMEOUT_MS) {
            if do_search {
                if let Event::Key(key_event) = event {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(c) => state.search_string.push(c),
                            KeyCode::Backspace => {
                                state.search_string.pop();
                            }
                            KeyCode::Esc => {
                                state.search_string = String::new();
                                do_search = false;
                            }
                            KeyCode::Enter => {
                                do_search = false;
                            }
                            _ => {}
                        }
                        do_render = true;
                    }
                }
            } else {
                handle_branch_event(
                    event,
                    &mut state,
                    &mut do_run,
                    &mut do_render,
                    &mut do_search,
                );
            }
        }
    }
    term.close();
}

fn render_debug_info(term: &mut Term, state: &mut State, args: &Args) {
    state.renders += 1;
    let term_size = Term::size();
    let x = term_size.x - 20 - PADDING as u16;
    let y = term_size.y - 1 - PADDING as u16;
    term.draw_text_bubble(
        Vec2::new(x, y - 2),
        format!(
            "Renders: {}\nSum len: {}\nSize:    {}",
            state.renders,
            args.summary_length,
            Term::size()
        ),
    );
}

fn render_branches(term: &mut Term, state: &mut State, args: &Args) {
    state.branches = get_branches(&state.repo)
        .into_iter()
        .filter(|b| {
            if state.search_string.is_empty() {
                true;
            }
            b.name
                .to_lowercase()
                .contains(&state.search_string.to_lowercase())
        })
        .collect();

    if state.selected_row > state.branches.len() {
        state.selected_row = state.branches.len() - 1
    }
    let longest_name = {
        let mut n = 0;
        for branch in state.branches.iter() {
            let challenge = branch.name.len();
            if challenge > n {
                n = challenge;
            }
        }
        n
    };
    let max_y = (Term::size().y - 1) as usize - PADDING;
    term.clear_all();
    if state.branches.len() == 0 {
        term.set_fg_color(Color::Grey);
        term.set_attribute(Attribute::Dim);
        term.write_text(Vec2::from((PADDING, max_y)), "> No branches found");
        term.reset_colors();
        term.reset_attributes();
        return;
    }

    for (i, branch) in state.branches.iter().enumerate() {
        let prefix = if i == state.selected_row { ">" } else { " " };
        if i == state.selected_row {
            term.set_attribute(Attribute::Bold);
        }
        if branch.is_head {
            term.set_fg_color(Color::DarkGreen);
        }
        if branch.is_gone {
            term.set_attribute(Attribute::CrossedOut);
        }

        let branch_summary = {
            let s = branch.summary.chars().take(args.summary_length).collect();
            if branch.summary.chars().count() > args.summary_length {
                format!("{s}...")
            } else {
                s
            }
        };
        let main_str = format!(
            "{prefix} {} {:<width$}  '{branch_summary}'",
            branch.short_oid(),
            branch.name,
            width = longest_name
        );
        let mut cursor_x = PADDING + main_str.len();

        term.write_text(Vec2::from((PADDING, max_y - i)), main_str);

        term.set_fg_color(Color::Grey);
        term.set_attribute(Attribute::Dim);

        if !branch.has_upstream {
            let msg = "\t[no upstream]";
            term.write_text(Vec2::from((cursor_x, max_y - i)), msg);
            cursor_x += msg.len();
        }
        if branch.is_gone {
            let msg = "\t[gone]";
            term.write_text(Vec2::from((cursor_x, max_y - i)), msg);
        }

        term.reset_attributes();
        term.reset_colors();
    }
}

fn handle_branch_event(
    event: Event,
    state: &mut State,
    do_run: &mut bool,
    do_render: &mut bool,
    do_search: &mut bool,
) {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) => *do_run = false,
        Event::Resize(_w, _h) => *do_render = true,

        // Movement
        Event::Key(KeyEvent {
            code: KeyCode::Char('k'),
            ..
        }) => {
            let n_branches = state.branches.len();
            if n_branches != 0 {
                if state.selected_row == n_branches - 1 {
                    state.selected_row = 0;
                } else {
                    state.selected_row += 1;
                }
                *do_render = true;
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('j'),
            ..
        }) => {
            let n_branches = state.branches.len();
            if n_branches != 0 {
                if state.selected_row == 0 {
                    state.selected_row = state.branches.len() - 1;
                } else {
                    state.selected_row -= 1;
                }
                *do_render = true;
            }
        }

        // Actions
        Event::Key(KeyEvent {
            code: KeyCode::Char('l'),
            ..
        }) => {
            if state.branches.len() != 0 {
                let selected_branch_name = &state.branches[state.selected_row].name;
                checkout_branch(&state.repo, selected_branch_name).unwrap();
                *do_render = true;
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Char('/'),
            ..
        }) => {
            *do_search = true;
            *do_render = true;
        }

        _ => {}
    }
}
