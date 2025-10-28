#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ShotgunAnimation {
    #[default]
    Idle = 0,
    Fire,
    Fire2,
    Reload,
    Pump,
    StartReload,
    Draw,
    Holster,
    Idle4,
    IdleDeep,
}
