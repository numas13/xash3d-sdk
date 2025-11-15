use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
    pub struct KeyState: i32 {
        const NONE          = 0;
        const DOWN          = 1 << 0;
        const IMPULSE_DOWN  = 1 << 1;
        const IMPULSE_UP    = 1 << 2;

        const ANY_DOWN      = Self::DOWN.union(Self::IMPULSE_DOWN).bits();
    }
}
