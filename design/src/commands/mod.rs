//! Command implementations

pub mod add;
pub mod add_headers;
pub mod debug;
pub mod index;
pub mod info;
pub mod list;
pub mod new;
pub mod remove;
pub mod replace;
pub mod scan;
pub mod search;
pub mod show;
pub mod sync_location;
pub mod transition;
pub mod update_index;
pub mod validate;

pub use add::{add_batch, add_document, preview_add};
pub use add_headers::add_headers;
pub use debug::{
    show_checksums, show_diff, show_document_state, show_orphans, show_state, show_stats,
    verify_document,
};
pub use index::generate_index;
pub use list::list_documents_with_state;
pub use new::new_document;
pub use remove::execute as remove_document;
pub use replace::execute as replace_document;
pub use scan::scan_documents;
pub use search::search;
pub use show::show_document;
pub use sync_location::sync_location;
pub use transition::transition_document;
pub use update_index::update_index;
pub use validate::validate_documents;
