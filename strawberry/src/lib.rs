pub mod logger;
mod search;
pub mod structs;
mod table;
use structs::{SearchCache, SearchParams, SearchResults};

/// Perform a full engine search for the best move with the given parameters and cache.
pub fn search(params: SearchParams<'_>, cache: &mut SearchCache) -> SearchResults {
    search::search(params, cache)
}
