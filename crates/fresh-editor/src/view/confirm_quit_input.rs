//! Input handler for ConfirmQuit / ConfirmQuitWithModified prompts.
//!
//! These quit-confirmation prompts are single-key modal dialogs: pressing
//! a character submits the prompt immediately without requiring Enter, and
//! Esc cancels. This matches the familiar y/n behaviour from nano/vim
//! that issue #546 asks for.
//!
//! The actual interpretation of the submitted character lives in
//! `app::prompt_actions::handle_confirm_quit{,_modified}` — this handler
//! is only responsible for routing keystrokes to the existing submit
//! path with the single typed character as input (via
//! `Action::PromptConfirmWithText`).

use crate::input::handler::{DeferredAction, InputContext, InputHandler, InputResult};
use crate::input::keybindings::Action;
use crossterm::event::{KeyCode, KeyEvent};

pub struct ConfirmQuitInputHandler;

impl ConfirmQuitInputHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConfirmQuitInputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler for ConfirmQuitInputHandler {
    fn handle_key_event(&mut self, event: &KeyEvent, ctx: &mut InputContext) -> InputResult {
        match event.code {
            // Submit the prompt with the typed character — no Enter needed.
            KeyCode::Char(c) => {
                ctx.defer(DeferredAction::ExecuteAction(
                    Action::PromptConfirmWithText(c.to_string()),
                ));
                InputResult::Consumed
            }
            // Enter on an empty prompt still cancels (existing semantics).
            KeyCode::Enter => {
                ctx.defer(DeferredAction::ConfirmPrompt);
                InputResult::Consumed
            }
            KeyCode::Esc => {
                ctx.defer(DeferredAction::ClosePrompt);
                InputResult::Consumed
            }
            _ => InputResult::Consumed,
        }
    }

    fn is_modal(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn char_key_submits_prompt_with_that_char() {
        let mut handler = ConfirmQuitInputHandler::new();
        let mut ctx = InputContext::new();

        let result = handler.handle_key_event(&key(KeyCode::Char('n')), &mut ctx);
        assert!(matches!(result, InputResult::Consumed));
        assert_eq!(ctx.deferred_actions.len(), 1);
        match &ctx.deferred_actions[0] {
            DeferredAction::ExecuteAction(Action::PromptConfirmWithText(text)) => {
                assert_eq!(text, "n");
            }
            other => panic!("expected PromptConfirmWithText, got {other:?}"),
        }
    }

    #[test]
    fn esc_closes_prompt() {
        let mut handler = ConfirmQuitInputHandler::new();
        let mut ctx = InputContext::new();

        let result = handler.handle_key_event(&key(KeyCode::Esc), &mut ctx);
        assert!(matches!(result, InputResult::Consumed));
        assert!(matches!(
            ctx.deferred_actions.as_slice(),
            [DeferredAction::ClosePrompt]
        ));
    }

    #[test]
    fn enter_confirms_with_existing_input() {
        let mut handler = ConfirmQuitInputHandler::new();
        let mut ctx = InputContext::new();

        let result = handler.handle_key_event(&key(KeyCode::Enter), &mut ctx);
        assert!(matches!(result, InputResult::Consumed));
        assert!(matches!(
            ctx.deferred_actions.as_slice(),
            [DeferredAction::ConfirmPrompt]
        ));
    }

    #[test]
    fn arrow_keys_consumed_silently() {
        let mut handler = ConfirmQuitInputHandler::new();
        let mut ctx = InputContext::new();

        let result = handler.handle_key_event(&key(KeyCode::Up), &mut ctx);
        assert!(matches!(result, InputResult::Consumed));
        assert!(ctx.deferred_actions.is_empty());
    }

    #[test]
    fn is_modal() {
        let handler = ConfirmQuitInputHandler::new();
        assert!(handler.is_modal());
    }
}
