#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GaussAnimation {
    #[default]
    Idle = 0,
    Idle2,
    Fidget,
    Spinup,
    Spin,
    Fire,
    Fire2,
    Holster,
    Draw,
}
