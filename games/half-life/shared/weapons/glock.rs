#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GlockAnimation {
    #[default]
    Idle1 = 0,
    Idle2,
    Idle3,
    Shoot,
    ShootEmpty,
    Reload,
    ReloadNotEmpty,
    Draw,
    Holster,
    AddSilencer,
}
