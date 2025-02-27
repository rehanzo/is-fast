use crate::errors::error::MyError;
use crate::links::link::Link;
use crate::tui::render;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;
use std::result::Result;
use open;

pub fn handle_input(
    index: &mut usize,
    links: &[Link],
    page: &mut Paragraph<'static>,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    scroll_offset: &mut u16,
    page_height: u16,
) -> Result<bool, MyError> {
    if let event::Event::Key(KeyEvent {
        code, modifiers, ..
    }) = event::read()?
    {
        match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('n') | KeyCode::Right if *index < links.len() - 1 => {
                *index += 1;
                change_page(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Char('b') | KeyCode::Left if *index > 0 => {
                *index -= 1;
                change_page(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                *scroll_offset = *scroll_offset + 1;
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                *scroll_offset = scroll_offset.saturating_sub(1); // Scroll up
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = scroll_offset.saturating_sub(page_height / 2);
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = *scroll_offset + (page_height / 2);
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Char('o') => {
                open_link(index, links);
            }
            _ => {}
        }
    }
    Ok(false)
}

fn open_link(index: &mut usize, links: &[Link]) {
    links
        .get(*index)
        .map(|link| format!("https://{}", link.url))
        .and_then(|url| open::that(&url).err())
        .iter()
        .for_each(|e| println!("{}", e));
}

fn change_page(
    index: &mut usize,
    links: &[Link],
    page: &mut Paragraph,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    scroll_offset: &mut u16,
) -> Result<(), MyError> {
    render::loading(terminal)?;
    *page = new_page(index, links);
    *scroll_offset = 0;
    draw(index, links, page, terminal, scroll_offset)?;
    Ok(())
}

fn new_page(index: &mut usize, links: &[Link]) -> Paragraph<'static> {
    links
        .get(*index)
        .map(|link| link.get_content())
        .unwrap_or_else(|| Paragraph::new(Text::from(String::from("Index out of bounds"))))
}

fn draw(
    index: &mut usize,
    links: &[Link],
    page: &mut Paragraph,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    scroll_offset: &mut u16,
) -> Result<(), MyError> {
    render::page(terminal, page, links.get(*index), *scroll_offset)
        .map_err(|e| MyError::DisplayError(e.to_string()))?;
    Ok(())
}
