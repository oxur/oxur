// Transport layer for REPL communication
//
// Provides abstraction over different connection types with async I/O.

mod traits;

// Re-export public types
pub use traits::{
    SplitTransport, Transport, TransportError, TransportReader, TransportWriter,
};
