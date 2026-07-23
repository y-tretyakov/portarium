use portarium_core::PortEvent;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    Tick,
    Scan,
    ScanComplete(Vec<PortEvent>),
    SelectUp,
    SelectDown,
    Enter,
    Back,
    Kill(u16, u32),
    Restart(u16, u32, String, String),
    ToggleHelp,
    ChangeScreen(Screen),
    Quit,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Dashboard,
    Detail,
    Graph,
}
