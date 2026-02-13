#[derive(Debug, Clone)]
pub struct EthicsContext {
    pub flags: Vec<String>,
    pub life_harm_flag: bool,
}

impl EthicsContext {
    pub fn is_clean(&self) -> bool {
        !self.life_harm_flag && self.flags.is_empty()
    }
}
