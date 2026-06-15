//! State representation for AISE.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lane {
    pub hi: u64,
    pub lo: u64,
}

impl Lane {
    #[inline(always)]
    pub const fn new(hi: u64, lo: u64) -> Self {
        Self { hi, lo }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub lanes: [Lane; 128],
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            lanes: [Lane::new(0, 0); 128],
        }
    }
}
