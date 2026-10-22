//! Versioned workspace configuration and language-independent identifiers.
//!
//! JSON field names and enum values are stable API identifiers. User-visible
//! names, spreadsheet headers, and filenames can use any language. Configured
//! names are literal strings; interface and output captions use locale catalogs.

mod indicators;
mod inputs;
mod validate;

pub use indicators::{Breakdown, Indicator, MetricClass};
pub use inputs::{DocumentRule, InputRules};

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

/// Current workspace schema: literal names, a separate roster, and shared input rules.
pub const SCHEMA_VERSION: u32 = 2;

/// How numeric observations are combined. Means are weighted by observations,
/// never calculated as the mean of project means.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aggregation {
    /// Add all present values.
    Sum,
    /// Arithmetic mean of present values.
    Mean,
    /// Smallest present value.
    Min,
    /// Largest present value.
    Max,
    /// Number of present numeric cells, including explicit zeros.
    Count,
}

/// Policy for an empty KPI cell in a nonempty data row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Missing {
    /// Stop with a diagnostic identifying the workbook, row, and column.
    #[default]
    Error,
    /// Substitute an observed zero (also contributes to mean and count).
    Zero,
    /// Exclude the cell from this metric's aggregation.
    Skip,
}

/// A configurable numeric KPI read from the annex worksheet.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    /// Unique portable identifier, independent of the display language.
    pub id: String,
    /// Display name, used unchanged in every interface and output language.
    pub label: String,
    /// Exact worksheet column heading, ignoring surrounding whitespace.
    pub column: String,
    /// Aggregation at project, group, category, and overall levels.
    pub aggregation: Aggregation,
    /// Handling of missing cells; defaults to `error`.
    #[serde(default)]
    pub missing: Missing,
    /// Optional unit displayed alongside the label.
    #[serde(default)]
    pub unit: String,
    /// Display precision (0–6); calculations retain the original precision.
    #[serde(default)]
    pub decimals: u8,
    /// Optional breakdown of this KPI by a text property in the same Excel row.
    /// The total and each class use this metric's aggregation and precision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Breakdown>,
}

/// A named group or project category, independent of project parent links.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Classification {
    /// Unique portable identifier.
    pub id: String,
    /// User-visible name, independent of the interface and output language.
    pub label: String,
}

/// One project in the separate roster. Input paths come from [`InputRules`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Unique portable identifier, also used for the output filename.
    pub id: String,
    /// Project title, independent of the interface and output language.
    pub label: String,
    /// Optional identifier of an existing group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Optional category identifier, independent of the group and parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Optional parent project ID. This is organizational metadata, not an
    /// instruction to add the parent's figures to its children's figures again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

/// The common worksheet layout used by every project's annex.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annex {
    /// Worksheet name in the source workbooks, in any language.
    pub sheet: String,
    /// One-based Excel row containing the headings.
    pub header_row: u32,
}

/// Complete portable report configuration. See the user guide for the JSON
/// schema, including ordering, numeric-cell, and path rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Schema version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Report title, used unchanged in every interface and output language.
    pub title: String,
    /// Language tag for generated captions and number formatting (for example
    /// `en` or `fr-CA`). Does not change configured names, Excel matching, or source PDF pages.
    pub locale: String,
    /// Free-text reporting period; REMAIN does not infer or filter dates.
    pub period: String,
    /// Shared worksheet layout, required only when a KPI workbook is defined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annex: Option<Annex>,
    /// Optional groups in central report order. Unassigned projects follow them.
    #[serde(default)]
    pub groups: Vec<Classification>,
    /// Optional project categories, in summary order.
    #[serde(default)]
    pub categories: Vec<Classification>,
    /// CSV or JSON project list, relative to the workspace. Row order determines
    /// project reading order within each group.
    pub projects: PathBuf,
    /// Shared directory and filename conventions for automatic input discovery.
    pub inputs: InputRules,
    /// Optional workspace-relative TrueType font for generated PDF contents pages.
    /// Defaults to bundled Noto Sans. Source PDF fonts are never changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contents_font: Option<PathBuf>,
    /// KPIs in display order. Empty when the workspace only collects PDFs.
    #[serde(default)]
    pub metrics: Vec<Metric>,
    /// Optional UI/output text overrides keyed like the bundled locale catalogs.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub translations: BTreeMap<String, String>,
}
