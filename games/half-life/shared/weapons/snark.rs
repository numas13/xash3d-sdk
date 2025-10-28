#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SqueakAnimation {
    #[default]
    Idle1 = 0,
    FidgetFit,
    FidgetNip,
    Down,
    Up,
    Throw,
}
