use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use crate::input::Command;

#[derive(Debug, Clone)]
pub struct KeyBindings {
    normal_mode: HashMap<KeyEvent, Command>,
    insert_mode: HashMap<KeyEvent, Command>,
    visual_mode: HashMap<KeyEvent, Command>,
    command_mode: HashMap<KeyEvent, Command>,
}

impl KeyBindings {
    pub fn new() -> Self {
        let mut keybindings = Self {
            normal_mode: HashMap::new(),
            insert_mode: HashMap::new(),
            visual_mode: HashMap::new(),
            command_mode: HashMap::new(),
        };

        keybindings.setup_default_bindings();
        keybindings
    }

    fn setup_default_bindings(&mut self) {
        self.bind_normal(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE), Command::MoveLeft);
        self.bind_normal(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE), Command::MoveDown);
        self.bind_normal(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE), Command::MoveUp);
        self.bind_normal(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE), Command::MoveRight);
        self.bind_normal(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE), Command::MoveWordForward);
        self.bind_normal(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE), Command::MoveWordBackward);
        self.bind_normal(KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE), Command::MoveLineStart);
        self.bind_normal(KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE), Command::MoveLineEnd);
        self.bind_normal(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE), Command::MoveFileStart);
        self.bind_normal(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE), Command::MoveFileEnd);

        self.bind_normal(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE), Command::EnterInsertMode);
        self.bind_normal(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE), Command::EnterInsertModeAfter);
        self.bind_normal(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE), Command::OpenLineBelow);
        self.bind_normal(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE), Command::EnterVisualMode);
        self.bind_normal(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE), Command::EnterCommandMode);

        self.bind_normal(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE), Command::DeleteChar);
        self.bind_normal(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE), Command::DeleteLine);

        self.bind_insert(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), Command::EnterNormalMode);

        self.bind_visual(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), Command::EnterNormalMode);

        self.bind_command(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), Command::EnterNormalMode);
        self.bind_command(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), Command::ExecuteCommand);
    }

    pub fn bind_normal(&mut self, key: KeyEvent, command: Command) {
        self.normal_mode.insert(key, command);
    }

    pub fn bind_insert(&mut self, key: KeyEvent, command: Command) {
        self.insert_mode.insert(key, command);
    }

    pub fn bind_visual(&mut self, key: KeyEvent, command: Command) {
        self.visual_mode.insert(key, command);
    }

    pub fn bind_command(&mut self, key: KeyEvent, command: Command) {
        self.command_mode.insert(key, command);
    }

    pub fn get_normal_binding(&self, key: &KeyEvent) -> Option<&Command> {
        self.normal_mode.get(key)
    }

    pub fn get_insert_binding(&self, key: &KeyEvent) -> Option<&Command> {
        self.insert_mode.get(key)
    }

    pub fn get_visual_binding(&self, key: &KeyEvent) -> Option<&Command> {
        self.visual_mode.get(key)
    }

    pub fn get_command_binding(&self, key: &KeyEvent) -> Option<&Command> {
        self.command_mode.get(key)
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}
