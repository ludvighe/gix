use crate::{
    branch::{checkout_branch, get_branches},
    term::{Term, Vec2},
};
use clap::Parser;
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
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

#[derive(Default)]
struct State {
    renders: usize,
}

fn main() {
    let args = Args::parse();
    let mut term = Term::new();
    let mut state = State::default();
    let mut do_run = true;
    let mut do_render = true;

    term.clear_all();

    let directory = Path::new(&args.directory);

    let repo = match Repository::open(directory) {
        Ok(repo) => repo,
        Err(_) => {
            exit(1);
        }
    };

    let mut branches = get_branches(&repo);
    let mut selected_row = 0;

    while do_run {
        if do_render {
            branches = get_branches(&repo);
            let longest_name = {
                let mut n = 0;
                for branch in branches.iter() {
                    let challenge = branch.name.len();
                    if challenge > n {
                        n = challenge;
                    }
                }
                n
            };
            let max_y = (Term::size().y - 1) as usize - PADDING;

            term.clear_all();
            if args.debug {
                render_debug_info(&mut term, &mut state, &args);
            }

            for (i, branch) in branches.iter().enumerate() {
                let prefix = if i == selected_row { ">" } else { " " };
                if i == selected_row {
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

            if branches.len() == 0 {
                term.write_text(Vec2::from((PADDING, PADDING)), "no branches in repo");
            }
            do_render = false;
        }
        if let Some(event) = term.read_event(EVENT_POLL_TIMEOUT_MS) {
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
                }) => do_run = false,
                Event::Resize(_w, _h) => do_render = true,

                // Movement
                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => {
                    let n_branches = branches.len();
                    if n_branches != 0 {
                        if selected_row == n_branches - 1 {
                            selected_row = 0;
                        } else {
                            selected_row += 1;
                        }
                        do_render = true;
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => {
                    let n_branches = branches.len();
                    if n_branches != 0 {
                        if selected_row == 0 {
                            selected_row = branches.len() - 1;
                        } else {
                            selected_row -= 1;
                        }
                        do_render = true;
                    }
                }

                // Actions
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    ..
                }) => {
                    if branches.len() != 0 {
                        let selected_branch_name = &branches[selected_row].name;
                        checkout_branch(&repo, selected_branch_name).unwrap();
                        do_render = true;
                    }
                }

                _ => {}
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
    term.write_text(Vec2::new(x, y - 2), format!("Renders: {}", state.renders));
    term.write_text(
        Vec2::new(x, y - 1),
        format!("Sum len: {}", args.summary_length),
    );
    term.write_text(Vec2::new(x, y), format!("Size:    {}", Term::size()));
}
