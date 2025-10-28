#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TripmineAnimation {
    #[default]
    Idle1 = 0,
    Idle2,
    Arm1,
    Arm2,
    Fidget,
    Holster,
    Draw,
    World,
    Ground,
}
