use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, Validation,
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

use std::sync::LazyLock;

const JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// Firebase ID token claims.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FirebaseClaims {
    /// Subject: Firebase UID
    pub sub: String,
    /// Audience: Firebase project ID
    pub aud: String,
    /// Issuer
    pub iss: String,
    /// Expiration (Unix timestamp)
    pub exp: u64,
    /// Issued at (Unix timestamp)
    pub iat: u64,
    /// Auth time (Unix timestamp)
    pub auth_time: u64,
    /// Email (present for email/password and Google sign-in)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Email verified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Provider data sign-in method ("google.com", "password", etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firebase: Option<FirebaseInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FirebaseInfo {
    pub sign_in_provider: String,
}

struct JwksCache {
    jwks: JwkSet,
    fetched_at: Instant,
}

static JWKS_CACHE: tokio::sync::OnceCell<RwLock<Option<JwksCache>>> =
    tokio::sync::OnceCell::const_new();

async fn get_jwks_lock() -> &'static RwLock<Option<JwksCache>> {
    JWKS_CACHE
        .get_or_init(|| async { RwLock::new(None) })
        .await
}

/// Fetch or return cached JWK set. Keys are refreshed every hour.
async fn fetch_jwks() -> Result<JwkSet, String> {
    let lock = get_jwks_lock().await;

    // Try to return from cache (read lock).
    {
        let cache = lock.read().await;
        if let Some(ref c) = *cache {
            if c.fetched_at.elapsed() < Duration::from_secs(3600) {
                return Ok(c.jwks.clone());
            }
        }
    }

    // Refresh (write lock).
    let mut cache = lock.write().await;
    // Double-check after acquiring write lock.
    if let Some(ref c) = *cache {
        if c.fetched_at.elapsed() < Duration::from_secs(3600) {
            return Ok(c.jwks.clone());
        }
    }

    let jwks: JwkSet = HTTP_CLIENT
        .get(JWKS_URL)
        .send()
        .await
        .map_err(|e| format!("failed to fetch JWKS: {e}"))?
        .json()
        .await
        .map_err(|e| format!("failed to parse JWKS: {e}"))?;

    *cache = Some(JwksCache {
        jwks: jwks.clone(),
        fetched_at: Instant::now(),
    });

    Ok(jwks)
}

/// Verify a Firebase ID token and return its claims.
///
/// Validates:
/// - RS256 signature against Google's public keys
/// - `iss` = `https://securetoken.google.com/<project_id>`
/// - `aud` = `<project_id>`
/// - `exp` and `iat`
pub async fn verify_firebase_token(token: &str) -> Result<FirebaseClaims, String> {
    let project_id = env::var("FIREBASE_PROJECT_ID")
        .map_err(|_| "FIREBASE_PROJECT_ID env var not set".to_string())?;

    let header = decode_header(token).map_err(|e| format!("invalid token header: {e}"))?;

    let kid = header
        .kid
        .ok_or_else(|| "token missing 'kid' header".to_string())?;

    let jwks = fetch_jwks().await?;

    let jwk = jwks
        .find(&kid)
        .ok_or_else(|| format!("no JWK found for kid '{kid}'"))?;

    let decoding_key = match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
            .map_err(|e| format!("failed to build decoding key: {e}"))?,
        _ => return Err("unsupported JWK algorithm".to_string()),
    };

    let expected_issuer = format!("https://securetoken.google.com/{project_id}");

    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[&project_id]);
    validation.set_issuer(&[&expected_issuer]);

    let token_data = decode::<FirebaseClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("token validation failed: {e}"))?;

    Ok(token_data.claims)
}
