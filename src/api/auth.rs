use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::post,
    Router,
};
use mongodb::bson::{doc, DateTime};
use serde_json::{json, Value};

use crate::services::{firebase, mongo};

pub fn router() -> Router {
    Router::new()
        .route("/me", post(me))
}

/// Extractor that validates a Firebase Bearer token and returns the verified claims.
pub struct FirebaseAuth(pub firebase::FirebaseClaims);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for FirebaseAuth {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .unwrap_or("");

        if token.is_empty() {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "missing Authorization header"})),
            )
                .into_response());
        }

        match firebase::verify_firebase_token(token).await {
            Ok(claims) => Ok(FirebaseAuth(claims)),
            Err(e) => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": e})),
            )
                .into_response()),
        }
    }
}

/// POST /api/auth/me
///
/// Requires a valid Firebase ID token in the `Authorization: Bearer <token>` header.
/// Looks up the customer record associated with the Firebase UID. If none exists,
/// creates one automatically using the profile information from the token claims.
/// Returns the customer document.
async fn me(FirebaseAuth(claims): FirebaseAuth) -> Json<Value> {
    let uid = &claims.sub;

    // Try to find an existing customer linked to this Firebase UID.
    let filter = doc! { "firebase_uid": uid };
    match mongo::find_one_by_filter("customers", filter).await {
        Ok(Some(doc)) => return Json(json!({"data": doc})),
        Err(e) => return Json(json!({"error": e})),
        Ok(None) => {}
    }

    // No existing customer — create one from the token claims.
    // Google and email/password sign-in always provide an email; other providers may not.
    let email = match claims.email {
        Some(ref e) if !e.is_empty() => e.clone(),
        _ => {
            return Json(json!({"error": "Firebase token does not contain an email address"}));
        }
    };
    let name = claims.name.unwrap_or_else(|| email.clone());
    let now = DateTime::now();

    let new_customer = doc! {
        "firebase_uid": uid,
        "name": name,
        "email": email,
        "active": true,
        "created_at": now,
        "updated_at": now,
    };

    match mongo::insert_one("customers", new_customer).await {
        Ok(result) => {
            // Fetch the newly created document to return it.
            if let Some(id) = result.get("inserted_id").and_then(|v| v.as_object())
                .and_then(|o| o.get("$oid"))
                .and_then(Value::as_str)
            {
                if let Ok(Some(doc)) = mongo::find_one("customers", id).await {
                    return Json(json!({"data": doc}));
                }
            }
            Json(json!({"data": result}))
        }
        Err(e) => Json(json!({"error": e})),
    }
}
