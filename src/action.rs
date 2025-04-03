#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Char(char),
    Backspace,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    LeftControlArrow,
    RightControlArrow,
    UpControlArrow,
    DownControlArrow,
    Escape,
    Enter,
    Paste(String),
    Copy,
}
