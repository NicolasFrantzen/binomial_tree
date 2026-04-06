//! Strategy-based customization for binomial tree evaluation.
//!
//! This module provides strategy traits and implementations for:
//! - **Leaf Smoothing** - How to value terminal nodes in the tree
//! - **Border Truncation** - Which boundary nodes to include or exclude

pub mod border_truncation;
pub mod leaf_smoothing;

// Re-export commonly used items
pub use border_truncation::ValueAtBorder;
pub use leaf_smoothing::ValueAtLeaf;
