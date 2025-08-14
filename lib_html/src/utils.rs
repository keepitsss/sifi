
use std::fmt::Display;

pub struct Indentation {
    pub level: u32,
    pub width: u8,
}
impl Default for Indentation {
    fn default() -> Self {
        Self { level: 0, width: 2 }
    }
}
impl Display for Indentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            " ".repeat((self.level * self.width as u32) as usize)
        )
    }
}
