//! Rename group dialog

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::*;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use super::DialogResult;
use crate::tui::components::render_text_field;
use crate::tui::styles::Theme;

pub struct GroupRenameDialog {
   current_name: String,
   input: Input,
   siblings: Vec<String>,
   error: Option<String>,
}

impl GroupRenameDialog {
   pub fn new(
      current_name: &str,
      siblings: Vec<String>,
   ) -> Self {
      Self {
         current_name: current_name.to_string(),
         input: Input::new(current_name.to_string()),
         siblings,
         error: None,
      }
   }

   pub fn handle_key(
      &mut self,
      key: KeyEvent,
   ) -> DialogResult<String> {
      match key.code {
         KeyCode::Esc => DialogResult::Cancel,
         KeyCode::Enter => {
            let new_name = self.input.value().trim().to_string();

            if new_name.is_empty() {
               return DialogResult::Cancel;
            }

            if new_name == self.current_name {
               return DialogResult::Cancel;
            }

            if new_name.contains('/') {
               self.error = Some("Group names cannot contain '/'".to_string());
               return DialogResult::Continue;
            }

            let new_name_lower = new_name.to_lowercase();
            let current_lower = self.current_name.to_lowercase();

            for sibling in &self.siblings {
               if sibling.to_lowercase() == new_name_lower && current_lower != new_name_lower {
                  self.error = Some("A group with that name already exists".to_string());
                  return DialogResult::Continue;
               }
            }

            DialogResult::Submit(new_name)
         },
         _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            self.error = None;
            DialogResult::Continue
         },
      }
   }

   pub fn render(
      &self,
      frame: &mut Frame,
      area: Rect,
      theme: &Theme,
   ) {
      let dialog_width = 50;
      let dialog_area = super::centered_rect(area, dialog_width, 12);

      frame.render_widget(Clear, dialog_area);

      let block = Block::default()
         .borders(Borders::ALL)
         .border_style(Style::default().fg(theme.accent))
         .title(" Rename Group ")
         .title_style(Style::default().fg(theme.title).bold());

      let inner = block.inner(dialog_area);
      frame.render_widget(block, dialog_area);

      let chunks = Layout::default()
         .direction(Direction::Vertical)
         .margin(1)
         .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
         ])
         .split(inner);

      let current_name_line = Line::from(vec![
         Span::styled("Current: ", Style::default().fg(theme.dimmed)),
         Span::styled(&self.current_name, Style::default().fg(theme.text)),
      ]);
      frame.render_widget(Paragraph::new(current_name_line), chunks[0]);

      render_text_field(frame, chunks[1], "New name:", &self.input, true, None, theme);

      if let Some(error) = &self.error {
         let error_line = Line::from(Span::styled(error.as_str(), Style::default().fg(Color::Red)));
         frame.render_widget(Paragraph::new(error_line), chunks[2]);
      }

      let footer = Line::from(vec![
         Span::styled("[Enter]", Style::default().fg(theme.accent)),
         Span::raw(" Rename  "),
         Span::styled("[Esc]", Style::default().fg(theme.accent)),
         Span::raw(" Cancel"),
      ]);
      frame.render_widget(Paragraph::new(footer).alignment(Alignment::Center), chunks[3]);
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_submit_valid_name() {
      let mut dialog = GroupRenameDialog::new("work", vec!["personal".to_string()]);
      dialog.input = Input::new("jobs".to_string());

      let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key_enter) {
         DialogResult::Submit(name) => assert_eq!(name, "jobs"),
         _ => panic!("Expected Submit"),
      }
   }

   #[test]
   fn test_cancel_on_esc() {
      let mut dialog = GroupRenameDialog::new("work", vec![]);
      let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Cancel => {},
         _ => panic!("Expected Cancel"),
      }
   }

   #[test]
   fn test_empty_submit_cancels() {
      let mut dialog = GroupRenameDialog::new("work", vec![]);
      dialog.input = Input::new(String::new());
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Cancel => {},
         _ => panic!("Expected Cancel"),
      }
   }

   #[test]
   fn test_same_name_cancels() {
      let mut dialog = GroupRenameDialog::new("work", vec![]);
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Cancel => {},
         _ => panic!("Expected Cancel"),
      }
   }

   #[test]
   fn test_duplicate_sibling_shows_error() {
      let mut dialog = GroupRenameDialog::new("work", vec!["personal".to_string()]);
      dialog.input = Input::new("personal".to_string());
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Continue => {
            assert_eq!(dialog.error, Some("A group with that name already exists".to_string()));
         },
         _ => panic!("Expected Continue with error"),
      }
   }

   #[test]
   fn test_slash_in_name_shows_error() {
      let mut dialog = GroupRenameDialog::new("work", vec![]);
      dialog.input = Input::new("my/group".to_string());
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Continue => {
            assert_eq!(dialog.error, Some("Group names cannot contain '/'".to_string()));
         },
         _ => panic!("Expected Continue with error"),
      }
   }

   #[test]
   fn test_error_cleared_on_input() {
      let mut dialog = GroupRenameDialog::new("work", vec!["personal".to_string()]);
      dialog.input = Input::new("personal".to_string());
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      dialog.handle_key(key);
      assert!(dialog.error.is_some());

      let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
      dialog.handle_key(key);
      assert!(dialog.error.is_none());
   }

   #[test]
   fn test_whitespace_trimmed() {
      let mut dialog = GroupRenameDialog::new("work", vec![]);
      dialog.input = Input::new("  jobs  ".to_string());
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Submit(name) => assert_eq!(name, "jobs"),
         _ => panic!("Expected Submit"),
      }
   }

   #[test]
   fn test_case_insensitive_sibling_conflict() {
      let mut dialog = GroupRenameDialog::new("work", vec!["personal".to_string()]);
      dialog.input = Input::new("Personal".to_string());
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Continue => {
            assert_eq!(dialog.error, Some("A group with that name already exists".to_string()));
         },
         _ => panic!("Expected Continue with error"),
      }
   }

   #[test]
   fn test_case_only_rename_allowed() {
      let mut dialog = GroupRenameDialog::new("Frontend", vec![]);
      dialog.input = Input::new("frontend".to_string());
      let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
      match dialog.handle_key(key) {
         DialogResult::Submit(name) => assert_eq!(name, "frontend"),
         _ => panic!("Expected Submit"),
      }
   }
}
