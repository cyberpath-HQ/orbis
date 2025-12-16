//! Static file serving and SPA fallback.

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use crate::state::AppState;

/// Create static files router.
pub fn router() -> Router<AppState> {
    Router::new()
        // SPA fallback - serve index.html for all unmatched routes
        .fallback(get(spa_fallback))
}

/// SPA fallback handler.
async fn spa_fallback() -> impl IntoResponse {
    // In production, this would serve the built React app's index.html
    // For now, return a simple HTML page
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Orbis</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
        }
        h1 {
            font-size: 3rem;
            margin-bottom: 0.5rem;
        }
        p {
            opacity: 0.8;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Orbis</h1>
        <p>Extensible Asset Management Platform</p>
        <p><small>Server is running. Build the frontend for the full experience.</small></p>
    </div>
</body>
</html>
"#)
}
