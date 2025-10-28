#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PythonAnimation {
    #[default]
    Idle1 = 0,
    Fidget,
    Fire1,
    Reload,
    Holster,
    Draw,
    Idle2,
    Idle3,
}
