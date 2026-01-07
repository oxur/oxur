// Transport layer for REPL communication
//
// Provides abstraction over different connection types with async I/O.

pub mod inprocess;
mod tcp;
mod traits;

// Re-export public types
pub use inprocess::{
    channel as inprocess_channel, InProcessClient, InProcessClientReader, InProcessClientWriter,
    InProcessServer, InProcessServerReader, InProcessServerWriter,
};
pub use tcp::{TcpTransport, TcpTransportListener, TcpTransportReader, TcpTransportWriter};
pub use traits::{SplitTransport, Transport, TransportError, TransportReader, TransportWriter};
