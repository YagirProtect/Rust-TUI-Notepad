#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Action{
    None,
    
    NewFile,
    SaveFile,
    OpenFile,
    
    Copy,
    Paste,
    Delete,
    Undo,
    Redo,
    Cut,
    Find,
    Replace,
    FAQ,
}