#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum HgunAnimation {
    #[default]
    Idle1 = 0,
    FidgetSway,
    FidgetShake,
    Down,
    Up,
    Shoot,
}
