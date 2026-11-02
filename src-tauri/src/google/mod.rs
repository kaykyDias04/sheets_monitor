//! Boundary reserved for Sprint 02 (SM-009/010/011).
//! This module does not perform requests or contain Google credentials.

/// Read-only provider contract to be implemented with explicit authorization later.
pub trait AuthorizedGoogleReader {
    type Spreadsheet;
    type Metadata;
    type ScriptProject;
    type Error;

    fn read_spreadsheet(&self, id: &str) -> Result<Self::Spreadsheet, Self::Error>;
    fn read_metadata(&self, id: &str) -> Result<Self::Metadata, Self::Error>;
    fn read_authorized_script(&self, id: &str) -> Result<Self::ScriptProject, Self::Error>;
}
