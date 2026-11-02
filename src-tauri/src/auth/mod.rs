//! Boundary reserved for Sprint 02 (SM-007/008).
//! No authorization endpoint, callback listener, credential or login is implemented.

/// Future desktop authorization boundary. A declaration is not an implementation.
pub trait DesktopAuthorization {
    type Error;

    /// Must later launch system-browser PKCE only after explicit user consent.
    fn connect(&self) -> Result<(), Self::Error>;

    /// Must later remove local credentials without modifying Google files.
    fn disconnect(&self) -> Result<(), Self::Error>;
}
