mod high_level_body;
mod typed_high_level_body;
mod typed_high_level_body_error;
mod typed_indexed_instr;

#[cfg(test)]
mod tests;

// pub use high_level_body::Body as HighLevelBody;
// pub use high_level_body::Instr as HighLevelInstr;

pub use high_level_body::LowLevelBody;

pub use typed_high_level_body::Body as HighLevelBody; // TypedIndexedHighLevelBody;
pub use typed_high_level_body::BodyInner;
pub use typed_high_level_body::Instr as HighLevelInstr;
pub use typed_high_level_body::TypedHighLevelInstr;

pub use typed_high_level_body_error::LowToHighError;
