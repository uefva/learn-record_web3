pub mod coin;
pub mod counter;
mod my_coin;
mod utility;

// pub use counter::process_instruction;
// pub use my_coin::process_instruction;
pub use coin::process_instruction;

solana_program::entrypoint!(process_instruction);
