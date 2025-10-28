#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum RpgAnimation {
    #[default]
    Idle = 0,
    Fidget,
    Reload,
    Fire2,
    Holster1,
    Draw1,
    Holster2,
    DrawUl,
    IdleUl,
    FidgetUl,
}
