#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mp5Animation {
    #[default]
    Longidle = 0,
    Idle1,
    Launch,
    Reload,
    Deploy,
    Fire1,
    Fire2,
    Fire3,
}
