//! Code shared by the API and the worker: the risk calculation, the figure
//! parser, the object-storage layout, what an error may say in a log, and a tiny
//! PDF writer used for demo documents.

pub mod figures;
pub mod logsafe;
pub mod pdfgen;
pub mod risk;
pub mod storage;
