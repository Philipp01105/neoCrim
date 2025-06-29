#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT", 
            Mode::Visual => "VISUAL",
            Mode::Command => "COMMAND",
        }
    }

    pub fn is_insert(&self) -> bool {
        matches!(self, Mode::Insert)
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Mode::Normal)
    }

    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual)
    }

    pub fn is_command(&self) -> bool {
        matches!(self, Mode::Command)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}
