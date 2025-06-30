#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // Movement commands
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,

    // Mode changes
    EnterNormalMode,
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterVisualMode,
    EnterCommandMode,

    // Text operations
    InsertChar(char),
    InsertNewline,
    DeleteChar,
    DeleteLine,
    OpenLineBelow,
    OpenLineAbove,

    // File operations
    Save,
    SaveAs(String),
    Open(String),
    Quit,
    ForceQuit,

    // Command execution
    ExecuteCommand,

    // Search
    Search(String),
    SearchNext,
    SearchPrevious,

    // Undo/Redo
    Undo,
    Redo,

    // Terminal
    OpenTerminal,
    ExecuteTerminalCommand(String),
    ToggleTerminal,

    // Theme commands
    SetTheme(String),
    SetThemeByIndex(usize),
    ListThemes,

    // No operation
    Noop,
}

impl Command {
    pub fn is_movement(&self) -> bool {
        matches!(self, 
            Command::MoveLeft | Command::MoveRight | Command::MoveUp | Command::MoveDown |
            Command::MoveWordForward | Command::MoveWordBackward |
            Command::MoveLineStart | Command::MoveLineEnd |
            Command::MoveFileStart | Command::MoveFileEnd
        )
    }

    pub fn is_edit(&self) -> bool {
        matches!(self,
            Command::InsertChar(_) | Command::InsertNewline |
            Command::DeleteChar | Command::DeleteLine |
            Command::OpenLineBelow | Command::OpenLineAbove
        )
    }

    pub fn is_mode_change(&self) -> bool {
        matches!(self,
            Command::EnterNormalMode | Command::EnterInsertMode |
            Command::EnterInsertModeAfter | Command::EnterVisualMode |
            Command::EnterCommandMode
        )
    }
}
