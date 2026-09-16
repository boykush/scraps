//! The on-disk Scraps IR: what the compiler knows about every scrap, kept
//! under `.scraps/` next to `.scraps.toml` so no command parses a source
//! the IR already covers. Regenerable, self-gitignored, never migrated.

pub mod hash;
pub mod link_table;
pub mod loader;
pub mod schema;
pub mod store;
