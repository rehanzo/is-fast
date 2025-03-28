use crate::database::connect::HistoryData;
use crate::tui::general_widgets::TUI_BORDER_COLOR;
use crate::tui::history::SearchOn;
use crate::tui::history::SearchOn::{Title, Url};
use chrono::{NaiveDateTime, Utc};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use once_cell::sync::Lazy;
use ratatui::layout::{Alignment, Constraint};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Cell, Paragraph, Row, Table};

static URL_COLOR: Lazy<&Style> = Lazy::new(crate::config::load::Config::get_url_color);
static TITLE_COLOR: Lazy<&Style> = Lazy::new(crate::config::load::Config::get_title_color);
static TIME_COLOR: Lazy<&Style> = Lazy::new(crate::config::load::Config::get_time_color);
pub static TEXT_COLOR: Lazy<&Style> = Lazy::new(crate::config::load::Config::get_text_color);

pub fn create_table<'a>(
    history: &[HistoryData],
    user_search: &str,
    search_on: &SearchOn,
) -> Table<'a> {
    let rows = create_rows(history, user_search, search_on);
    let table = Table::from_iter(rows)
        .widths([
            Constraint::Percentage(50),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ])
        .column_spacing(1)
        .highlight_symbol("> ")
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));
    table
}

fn create_rows(
    history: &[HistoryData],
    user_search: &str,
    search_on: &SearchOn,
) -> Vec<Row<'static>> {
    let rows: Vec<Row> = history
        .iter()
        .map(|h| match search_on {
            Title => {
                let cell = vec![
                    Cell::from(highlight_text(clip_if_needed(&h.title, 100), user_search))
                        .style(**TITLE_COLOR),
                    Cell::from(clip_if_needed(&h.url, 60)).style(**URL_COLOR),
                    Cell::from(date_to_display(&h.time)).style(**TIME_COLOR),
                ];
                Row::new(cell)
            }
            Url => {
                let cells = vec![
                    Cell::from(clip_if_needed(&h.title, 100)).style(**TITLE_COLOR),
                    Cell::from(highlight_text(clip_if_needed(&h.url, 60), user_search))
                        .style(**URL_COLOR),
                    Cell::from(date_to_display(&h.time)).style(**TIME_COLOR),
                ];
                Row::new(cells)
            }
        })
        .collect();
    rows
}

fn highlight_text(plain_text: String, user_search: &str) -> Line<'static> {
    let user_search = user_search.replace(' ', "");
    if user_search.is_empty() || plain_text.is_empty() {
        return Line::from(plain_text);
    }
    let mut matcher = Matcher::new(Config::DEFAULT);

    let mut indices = vec![];
    let mut binding1 = vec![];
    let mut binding2 = vec![];
    let haystack = Utf32Str::new(&plain_text, &mut binding1);
    let lowercase = user_search.to_lowercase(); // Panics with uppercase??? Lowercase still matches
    let needle = Utf32Str::new(&lowercase, &mut binding2);
    matcher.fuzzy_indices(haystack, needle, &mut indices);
    if indices.is_empty() {
        return Line::from(plain_text);
    }
    let mut idx = indices.remove(0);
    let mut current = String::new();
    let mut spans = vec![];
    let mut found = false;
    for (c, i) in plain_text.chars().zip(0..) {
        if found || i < idx {
            current.push(c);
        } else {
            spans.push(Span::from(current.clone()));
            current = String::new();
            spans.push(Span::styled(
                String::from(c),
                Style::from(Color::Red).add_modifier(Modifier::BOLD),
            ));
            if indices.is_empty() {
                found = true;
            } else {
                idx = indices.remove(0);
            }
        }
    }
    spans.push(Span::from(current));
    Line::from(spans)
}
fn clip_if_needed(text: &str, max_length: usize) -> String {
    if text.len() > max_length {
        return format!("{}...", &text[0..max_length - 3]);
    }
    text.to_string()
}

fn date_to_display(date: &NaiveDateTime) -> String {
    let duration = Utc::now().signed_duration_since(date.and_utc());
    if duration.num_weeks() > 0 {
        return format_time(duration.num_weeks(), "weeks");
    }
    if duration.num_days() > 0 {
        return format_time(duration.num_days(), "days");
    }
    if duration.num_hours() > 0 {
        return format_time(duration.num_hours(), "hours");
    }
    if duration.num_minutes() > 0 {
        return format_time(duration.num_minutes(), "minutes");
    }
    if duration.num_seconds() > 0 {
        return format_time(duration.num_seconds(), "seconds");
    }
    "Date could not be displayed".to_string()
}

fn format_time(amount: i64, time_measurement: &str) -> String {
    format!("{amount} {time_measurement} ago")
}

pub fn draw_search_text<'a>(user_input: &'a str, search_on: &'a SearchOn) -> Paragraph<'a> {
    let searched_on_text = searched_on_to_string(search_on);
    Paragraph::new(
        Line::from(format!(" [{searched_on_text}] {user_input}"))
            .style(TEXT_COLOR.add_modifier(Modifier::BOLD)),
    )
}
pub fn draw_history_count(row_count: u16) -> ratatui::prelude::Text<'static> {
    ratatui::prelude::Text::from(vec![
        Line::default(), // Move to the bottom line
        Line::from(count_result_text(row_count))
            .style(TUI_BORDER_COLOR.add_modifier(Modifier::BOLD))
            .alignment(Alignment::Right),
    ])
}

fn count_result_text(row_count: u16) -> String {
    if row_count == 1 {
        format!("{row_count} result ")
    } else {
        format!("{row_count} results ")
    }
}

fn searched_on_to_string(search_on: &SearchOn) -> String {
    match search_on {
        Title => String::from("TITLE"),
        Url => String::from("URL"),
    }
}
