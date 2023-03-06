use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::time::{Duration, Instant};
use std::{io, sync::mpsc, thread};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState,
    },
    Terminal,
};

mod db;
use crate::db::*;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_secs(1);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut task_state = TableState::default();
    task_state.select(None);

    let tasklists = read_db();
    let mut task_len = tasklists[0].tasks.len() - 1;

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);

            let title = Paragraph::new("Tasks But Good")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Red))
                        .border_type(BorderType::Double),
                );

            rect.render_widget(title, chunks[0]);
            let list_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[1]);
            let lists = render_lists(&tasklists);
            let selected_list = list_state
                .selected()
                .expect("There must be a selected list");
            let tasks = render_tasks(&tasklists[selected_list]);
            task_len = tasklists[selected_list].tasks.len() - 1;
            rect.render_stateful_widget(lists, list_chunks[0], &mut list_state);
            rect.render_stateful_widget(tasks, list_chunks[1], &mut task_state);
        })?;

        let list_len = tasklists.len() - 1;

        match rx.recv()? {
            Event::Input(event) => match task_state.selected() {
                Some(task_selected) => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Char('h') => {
                        task_state.select(None);
                    }
                    KeyCode::Char('j') => {
                        if task_selected != task_len {
                            task_state.select(Some(task_selected + 1));
                        }
                    }
                    KeyCode::Char('k') => {
                        if task_selected != 0 {
                            task_state.select(Some(task_selected as usize - 1));
                        }
                    }
                    _ => {}
                },
                None => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Char('j') => {
                        if let Some(selected) = list_state.selected() {
                            if selected != list_len {
                                list_state.select(Some(selected + 1));
                            }
                        }
                    }
                    KeyCode::Char('k') => {
                        if let Some(selected) = list_state.selected() {
                            if selected != 0 {
                                list_state.select(Some(selected - 1));
                            }
                        }
                    }
                    KeyCode::Char('l') => {
                        task_state.select(Some(0));
                    }
                    _ => {}
                },
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_lists<'a>(lists: &Vec<TaskList>) -> List<'a> {
    let tasks = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Lists")
        .border_type(BorderType::Plain);
    let lists: Vec<_> = lists
        .iter()
        .map(|list| {
            ListItem::new(Spans::from(vec![Span::styled(
                list.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    List::new(lists).block(tasks).highlight_style(
        Style::default()
            .bg(Color::Red)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}

fn render_tasks<'a>(list: &TaskList) -> Table<'a> {
    let tasks: Vec<Row> = (*list.tasks)
        .to_owned()
        .iter()
        .map(|task| {
            Row::new(vec![
                Cell::from(Span::raw(task.name.to_owned())),
                Cell::from(Span::raw(format!("{:?}", task.tags))),
                Cell::from(Span::raw(format!("{}", task.start_date.format("%D %T")))),
                Cell::from(Span::raw(format!("{}", task.due_date.format("%D %T")))),
            ])
        })
        .collect();

    let table = ["Name", "Tags", "Start Date", "Due Date"];

    let table = table
        .iter()
        .map(|t| {
            Cell::from(Span::styled(
                t.to_owned(),
                Style::default().add_modifier(Modifier::BOLD),
            ))
        })
        .collect::<Vec<Cell>>();

    let table = Table::new(tasks)
        .header(Row::new(table))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(list.name.to_owned())
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .highlight_style(
            Style::default()
                .bg(Color::Red)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    table
}
