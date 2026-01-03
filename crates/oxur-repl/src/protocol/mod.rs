// Oxur REPL Protocol
//
// Binary protocol for client-server REPL communication.
// Based on ODD-0018: Oxur Remote REPL Protocol Design.

mod codec;
mod messages;

// Re-export all public types
pub use codec::{CodecError, PostcardCodec};
pub use messages::{
    ErrorInfo, ErrorKind, HistoryEntry, MessageId, Operation, OperationResult, ReplMode, Request,
    Response, SessionId, SessionInfo, SourceLocation, Status,
};
