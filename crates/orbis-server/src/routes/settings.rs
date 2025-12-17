//! Settings routes.

use axum::{
    extract::State,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;

use crate::error::ServerResult;
use crate::extractors::AdminUser;
use crate::state::AppState;

/// Setting value from database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: Value,
    pub description: Option<String>,
    pub is_secret: bool,
}

/// Create settings router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings", get(get_settings))
        .route("/settings", put(update_settings))
}

/// Get application settings (admin only).
async fn get_settings(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> ServerResult<Json<Value>> {
    let db = state.db();
    
    let settings = match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            let rows = sqlx::query(
                "SELECT key, value, description, is_secret FROM settings ORDER BY key"
            )
            .fetch_all(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            rows.into_iter()
                .map(|row| Setting {
                    key: row.get("key"),
                    value: row.get("value"),
                    description: row.get("description"),
                    is_secret: row.get("is_secret"),
                })
                .collect::<Vec<_>>()
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            let rows = sqlx::query(
                "SELECT key, value, description, is_secret FROM settings ORDER BY key"
            )
            .fetch_all(pool)
            .await
            .map_err(|e| orbis_core::Error::database(e.to_string()))?;

            rows.into_iter()
                .map(|row| {
                    let value_str: String = row.get("value");
                    Setting {
                        key: row.get("key"),
                        value: serde_json::from_str(&value_str).unwrap_or(Value::Null),
                        description: row.get("description"),
                        is_secret: row.get("is_secret"),
                    }
                })
                .collect::<Vec<_>>()
        }
    };

    // Filter out secret values, only show that they exist
    let filtered_settings: Vec<Value> = settings
        .into_iter()
        .map(|s| {
            if s.is_secret {
                json!({
                    "key": s.key,
                    "value": "[REDACTED]",
                    "description": s.description,
                    "is_secret": true
                })
            } else {
                json!({
                    "key": s.key,
                    "value": s.value,
                    "description": s.description,
                    "is_secret": false
                })
            }
        })
        .collect();

    Ok(Json(json!({
        "success": true,
        "data": {
            "settings": filtered_settings
        }
    })))
}

/// Update settings request.
#[derive(Debug, Deserialize)]
struct UpdateSettingsRequest {
    settings: Vec<SettingUpdate>,
}

/// Setting update.
#[derive(Debug, Deserialize)]
struct SettingUpdate {
    key: String,
    value: Value,
    description: Option<String>,
    is_secret: Option<bool>,
}

/// Update application settings (admin only).
async fn update_settings(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(request): Json<UpdateSettingsRequest>,
) -> ServerResult<Json<Value>> {
    let db = state.db();

    match db.pool() {
        orbis_db::DatabasePool::Postgres(pool) => {
            for setting in request.settings {
                let is_secret = setting.is_secret.unwrap_or(false);
                sqlx::query(
                    r#"
                    INSERT INTO settings (key, value, description, is_secret, updated_at)
                    VALUES ($1, $2, $3, $4, NOW())
                    ON CONFLICT (key) DO UPDATE SET
                        value = EXCLUDED.value,
                        description = COALESCE(EXCLUDED.description, settings.description),
                        is_secret = COALESCE(EXCLUDED.is_secret, settings.is_secret),
                        updated_at = NOW()
                    "#
                )
                .bind(&setting.key)
                .bind(&setting.value)
                .bind(&setting.description)
                .bind(is_secret)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
        }
        orbis_db::DatabasePool::Sqlite(pool) => {
            for setting in request.settings {
                let is_secret = setting.is_secret.unwrap_or(false);
                let value_str = serde_json::to_string(&setting.value)
                    .map_err(|e| orbis_core::Error::database(e.to_string()))?;
                    
                sqlx::query(
                    r#"
                    INSERT OR REPLACE INTO settings (key, value, description, is_secret, updated_at)
                    VALUES ($1, $2, $3, $4, datetime('now'))
                    "#
                )
                .bind(&setting.key)
                .bind(&value_str)
                .bind(&setting.description)
                .bind(is_secret)
                .execute(pool)
                .await
                .map_err(|e| orbis_core::Error::database(e.to_string()))?;
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": "Settings updated"
    })))
}
