#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Error(String),
}
