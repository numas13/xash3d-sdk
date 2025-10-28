#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum CrowbarAnimation {
    #[default]
    Idle = 0,
    Draw,
    Holster,
    Attack1Hit,
    Attack1Miss,
    Attack2Miss,
    Attack2Hit,
    Attack3Miss,
    Attack3Hit,
}
