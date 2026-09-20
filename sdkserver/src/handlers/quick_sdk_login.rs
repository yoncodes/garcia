use axum::{Json, extract::State, http::StatusCode};
use sqlx::SqlitePool;

use crate::models::{QuickSdkLoginRequest, QuickSdkLoginResponse};

pub async fn login(
    State(database): State<SqlitePool>,
    Json(request): Json<QuickSdkLoginRequest>,
) -> Result<Json<QuickSdkLoginResponse>, StatusCode> {
    tracing::info!(
        uid = request.uid,
        token_present = !request.token.is_empty(),
        "QuickSDK login requested"
    );

    if let Ok(upstream) = std::env::var("GARCIA_CAPTURE_SDK_UPSTREAM") {
        let url = format!("{}/quickSDKlogin", upstream.trim_end_matches('/'));
        let response = reqwest::Client::new()
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(%error, %url, "SDK login forwarding failed");
                StatusCode::BAD_GATEWAY
            })?;
        if !response.status().is_success() {
            tracing::warn!(status = %response.status(), %url, "upstream SDK rejected login");
            return Err(StatusCode::BAD_GATEWAY);
        }
        let login = response
            .json::<QuickSdkLoginResponse>()
            .await
            .map_err(|error| {
                tracing::warn!(%error, %url, "invalid upstream SDK login response");
                StatusCode::BAD_GATEWAY
            })?;
        let account_state = account_state(&database, login.guid).await?;
        tracing::info!(
            guid = login.guid,
            node_id = login.node_id,
            bl_reg = login.bl_reg,
            account_state,
            %url,
            "upstream SDK login accepted"
        );
        return Ok(Json(login));
    }

    let guid = request.uid.parse::<u64>().map_err(|_| {
        tracing::warn!(uid = request.uid, "QuickSDK login rejected: invalid uid");
        StatusCode::BAD_REQUEST
    })?;
    if request.token.is_empty() {
        tracing::warn!(uid = request.uid, "QuickSDK login rejected: missing token");
        return Err(StatusCode::BAD_REQUEST);
    }
    let account_state = account_state(&database, guid).await?;

    tracing::info!(
        guid,
        node_id = "en-gs3",
        bl_reg = true,
        account_state,
        "QuickSDK login accepted"
    );
    Ok(Json(QuickSdkLoginResponse {
        code: 1,
        guid,
        node_id: "en-gs3".into(),
        token: format!("garcia-{guid}"),
        bl_reg: true,
    }))
}

async fn account_state(database: &SqlitePool, guid: u64) -> Result<&'static str, StatusCode> {
    let uid = i64::try_from(guid).map_err(|_| StatusCode::BAD_REQUEST)?;
    database::db::player_state::exists(database, uid)
        .await
        .map(|exists| if exists { "existing" } else { "new" })
        .map_err(|error| {
            tracing::error!(guid, %error, "failed to read SDK account state");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validates_and_accepts_login() {
        let database = database::connect_memory().await.unwrap();
        let Json(response) = login(
            State(database.clone()),
            Json(QuickSdkLoginRequest {
                uid: "3028256871".into(),
                token: "channel-token".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(response.guid, 3_028_256_871);
        assert_eq!(response.node_id, "en-gs3");
        assert!(response.bl_reg);

        let error = login(
            State(database),
            Json(QuickSdkLoginRequest {
                uid: "invalid".into(),
                token: String::new(),
            }),
        )
        .await
        .unwrap_err();
        assert_eq!(error, StatusCode::BAD_REQUEST);
    }
}
