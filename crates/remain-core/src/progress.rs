//! Coarse workflow progress for desktop or command-line integrations.

/// Current operation. Project numbers are one-based and describe the project
/// being read, not a percentage of total runtime. Callbacks run synchronously
/// on the caller's thread and should return promptly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    /// Find expected files and identify missing submissions.
    Discovering,
    /// Read and validate one complete project's PDFs and optional workbook.
    ReadingProject {
        /// One-based project number in report order.
        current: usize,
        /// Number of complete projects to read.
        total: usize,
    },
    /// Combine observations into project, group, category, and overall results.
    Summarizing,
    /// Create project PDFs and aggregate PDFs with contents pages.
    AssemblingPdfs,
    /// Write KPI results and submission status to Excel.
    WritingWorkbook,
    /// Write HTML, JSON, and metadata snapshots.
    WritingSummary,
    /// Publish the completed bundle to a new output directory.
    Publishing,
}
