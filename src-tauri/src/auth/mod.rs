//! SM-007/008: desktop PKCE, backend-only secrets and recoverable Windows credentials.
mod manager;
mod protocol;
mod transport;
mod vault;

pub use manager::{
    AccountIdentity, AuthConfigInput, AuthManager, AuthStatus, AuthorizedSession, IntegrationState,
};
pub use transport::{HttpsOAuthTransport, OAuthTransport, TokenGrant, TokenRequest, TokenResponse};
pub use vault::{CredentialVault, WindowsVault};

use serde::{Deserialize, Serialize};

pub const SHEETS_SCOPE: &str = "https://www.googleapis.com/auth/spreadsheets.readonly";
pub const SCRIPT_SCOPE: &str = "https://www.googleapis.com/auth/script.projects.readonly";
pub const DRIVE_SCOPE: &str = "https://www.googleapis.com/auth/drive.metadata.readonly";
pub const CLOUD_SCOPE: &str = "https://www.googleapis.com/auth/drive.appdata";
pub const AUDIT_SCOPE: &str = "https://www.googleapis.com/auth/admin.reports.audit.readonly";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthCode {
    Unconfigured,
    InvalidConfiguration,
    AuthBusy,
    Cancelled,
    ConsentDenied,
    CallbackInvalid,
    Timeout,
    Network,
    AuthorizationFailed,
    InvalidGrant,
    ReconnectRequired,
    InsufficientScope,
    VaultUnavailable,
    VaultCorrupt,
    OperationSuperseded,
    RevocationConfirmationRequired,
    IdentityMismatch,
    InvalidResponse,
    BrowserUnavailable,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct AuthError {
    pub code: AuthCode,
    pub message: &'static str,
    pub retryable: bool,
}

impl AuthError {
    pub fn new(code: AuthCode) -> Self {
        use AuthCode::*;
        let message = match code {
            Unconfigured => "Configure um cliente OAuth do tipo aplicativo para computador.",
            InvalidConfiguration => "Configuração OAuth inválida. Verifique o client ID e o client secret desktop.",
            AuthBusy => "Uma operação de autenticação está em andamento.",
            Cancelled => "Autenticação cancelada.",
            ConsentDenied => "O consentimento Google foi negado ou cancelado.",
            CallbackInvalid => "A resposta de autorização local foi recusada.",
            Timeout => "A operação excedeu o tempo disponível. Tente novamente.",
            Network => "Não foi possível conectar ao Google. A credencial protegida foi preservada.",
            AuthorizationFailed => "O Google recusou a autorização. Verifique o cliente desktop e tente conectar novamente.",
            InvalidGrant => "A autorização expirou ou foi revogada. Conecte a conta novamente.",
            ReconnectRequired => "Conecte a conta Google novamente para continuar.",
            InsufficientScope => "A permissão necessária não foi concedida. Revise os escopos e reconecte a conta.",
            VaultUnavailable => "O Windows Credential Manager não pôde concluir a operação. Nenhum fallback em texto simples foi usado.",
            VaultCorrupt => "A credencial protegida não pôde ser recuperada. Desconecte e configure a conexão novamente.",
            OperationSuperseded => "A sessão mudou durante a operação. O resultado anterior foi descartado.",
            RevocationConfirmationRequired => "Confirme explicitamente a revogação do acesso no Google.",
            IdentityMismatch => "A identidade retornada não corresponde à sessão. Reconecte a conta.",
            InvalidResponse => "O Google retornou uma resposta de autenticação inválida.",
            BrowserUnavailable => "Não foi possível abrir o navegador do sistema.",
        };
        Self {
            code,
            message,
            retryable: matches!(code, Timeout | Network | AuthBusy),
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}
impl std::error::Error for AuthError {}
pub type AuthResult<T> = Result<T, AuthError>;
