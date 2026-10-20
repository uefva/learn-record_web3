pub mod counter;
pub mod coin;
mod utility;

pub use counter::process_instruction;
// pub use coin::process_instruction;

solana_program::entrypoint!(process_instruction);