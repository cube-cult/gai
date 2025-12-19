pub mod commit;
pub mod diffs;
pub mod errors;
pub mod lines;
pub mod log;
pub mod patches;
pub mod repo;
pub mod settings;
pub mod staging;
pub mod status;
pub mod utils;

pub use diffs::Diffs;
pub use repo::GitRepo;
