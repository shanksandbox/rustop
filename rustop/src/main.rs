// standard lib imports
// input/output and time for refreshing 
use std::{
    io,
    time::{Duration, Instant},
};

// for terminal contorl 
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

// TUI framework
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, List, ListItem},
    Terminal,
};

// sys info and proc
use sysinfo::System;

fn main() -> Result<(), io::Error> {
    // disable line buffering, auto line buffering and detect keypresses
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // system obj for metric collection
    let mut sys = System::new_all();

    let tick_rate = Duration::from_millis(800);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            sys.refresh_all();

            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(5),
                ])
                .split(size);

            // CPU
            let cpu_usage = sys.global_cpu_info().cpu_usage();

            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(cpu_usage as u16);

            // Memory
            let mem_percent = if sys.total_memory() > 0 {
                ((sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0) as u16
            } else {
                0
            };

            let mem_gauge = Gauge::default()
                .block(Block::default().title("Memory").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(mem_percent);

            f.render_widget(cpu_gauge, chunks[0]);
            f.render_widget(mem_gauge, chunks[1]);

            // Processes
            let mut processes: Vec<_> = sys.processes().values().collect();

            processes.sort_by(|a, b| {
                b.cpu_usage()
                    .partial_cmp(&a.cpu_usage())
                    .unwrap()
            });

            let items: Vec<ListItem> = processes
                .iter()
                .take(10)
                .map(|p| {
                    ListItem::new(format!(
                        "PID: {:<6} | {:<20} | CPU: {:>5.1}% | MEM: {:>5} MB",
                        p.pid(),
                        p.name(),
                        p.cpu_usage(),
                        p.memory() / 1024
                    ))
                })
                .collect();

            let process_list = List::new(items)
                .block(Block::default().title("Top Processes").borders(Borders::ALL));

            f.render_widget(process_list, chunks[2]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // cleanup and back to terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}