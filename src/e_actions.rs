use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Action {
    None,

    NewFile,
    Exit,
    SaveFile,
    SaveFileAs,
    OpenFile,
    OpenInExplorer,
    OpenPath(PathBuf),
    RemoveRecentPath(PathBuf),
    SetFilesTabsScroll(u16),

    Copy,
    Paste,
    Delete,
    Undo,
    Redo,
    Cut,
    Find,
    Replace,
    ToggleKeywordHighlight,
    FAQ,
    OpenUrl(String),
}
