pub enum GmsgError {
    AiError,
    Tui,
    Git,
}

pub enum AiError {
    LimitExceeded,
    NetworkFailure,
    InvalidProvider,
    InvalidModel,
    InvalidApiKey,
}

pub enum GitError {}
