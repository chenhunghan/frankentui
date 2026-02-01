#![forbid(unsafe_code)]

//! Text handling for FrankenTUI.
//!
//! This crate provides text primitives for styled text rendering:
//! - [`Segment`] - atomic unit of styled text with cell-aware splitting
//! - [`SegmentLine`] - a line of segments
//! - [`SegmentLines`] - multi-line text
//!
//! # Example
//! ```
//! use ftui_text::{Segment, SegmentLine, split_into_lines};
//! use ftui_style::Style;
//!
//! // Create styled segments
//! let seg = Segment::styled("Error:", Style::new().bold());
//!
//! // Split text at cell boundaries
//! let (left, right) = Segment::text("hello world").split_at_cell(5);
//! assert_eq!(left.as_str(), "hello");
//!
//! // Handle multi-line text
//! let lines = split_into_lines(vec![
//!     Segment::text("line 1"),
//!     Segment::newline(),
//!     Segment::text("line 2"),
//! ]);
//! assert_eq!(lines.len(), 2);
//! ```

pub mod segment;

pub use segment::{ControlCode, Segment, SegmentLine, SegmentLines, join_lines, split_into_lines};
