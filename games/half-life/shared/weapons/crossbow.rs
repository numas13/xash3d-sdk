#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CrossbowAnimation {
    #[default]
    Idle1 = 0,
    Idle2,
    Fidget1,
    Fidget2,
    Fire1,
    Fire2,
    Fire3,
    Reload,
    Draw1,
    Draw2,
    Holster1,
    Holster2,
}
