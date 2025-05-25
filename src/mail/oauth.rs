use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use oauth2::{
    basic::BasicClient, AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

// Google OAuth2設定
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_REDIRECT_URI: &str = "http://localhost:8080/oauth/callback";

// Gmail API スコープ
const GMAIL_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";
const GMAIL_MODIFY_SCOPE: &str = "https://www.googleapis.com/auth/gmail.modify";
const GMAIL_SEND_SCOPE: &str = "https://www.googleapis.com/auth/gmail.send";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

pub struct GoogleOAuthClient {
    oauth_client: BasicClient,
    config: GoogleOAuthConfig,
    http_client: reqwest::Client,
}

impl GoogleOAuthClient {
    pub fn new(config: GoogleOAuthConfig) -> Result<Self> {
        let oauth_client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(GOOGLE_AUTH_URL.to_string())
                .context("Invalid authorization endpoint URL")?,
            Some(
                TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                    .context("Invalid token endpoint URL")?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone()).context("Invalid redirect URL")?,
        )
        .set_auth_type(AuthType::RequestBody);

        let http_client = reqwest::Client::new();

        Ok(Self {
            oauth_client,
            config,
            http_client,
        })
    }

    /// 認証URLを生成
    pub fn get_authorization_url(&self) -> (Url, CsrfToken) {
        self.oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(GMAIL_READONLY_SCOPE.to_string()))
            .add_scope(Scope::new(GMAIL_MODIFY_SCOPE.to_string()))
            .add_scope(Scope::new(GMAIL_SEND_SCOPE.to_string()))
            .url()
    }

    /// 認証コードをアクセストークンに交換
    pub async fn exchange_code_for_token(
        &self,
        authorization_code: String,
    ) -> Result<GoogleTokens> {
        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(authorization_code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to exchange authorization code for token")?;

        Ok(GoogleTokens {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|rt| rt.secret().clone()),
            expires_in: token_result.expires_in().map(|duration| duration.as_secs()),
            token_type: "Bearer".to_string(),
        })
    }

    /// リフレッシュトークンで新しいアクセストークンを取得
    pub async fn refresh_access_token(&self, refresh_token: String) -> Result<GoogleTokens> {
        let refresh_token = oauth2::RefreshToken::new(refresh_token);

        let token_result = self
            .oauth_client
            .exchange_refresh_token(&refresh_token)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to refresh access token")?;

        Ok(GoogleTokens {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|rt| rt.secret().clone()),
            expires_in: token_result.expires_in().map(|duration| duration.as_secs()),
            token_type: "Bearer".to_string(),
        })
    }

    /// ユーザー情報を取得
    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get user info: {}", response.status());
        }

        let user_info: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse user info response")?;

        Ok(GoogleUserInfo {
            email: user_info["email"].as_str().unwrap_or_default().to_string(),
            name: user_info["name"].as_str().unwrap_or_default().to_string(),
            picture: user_info["picture"].as_str().map(|s| s.to_string()),
        })
    }

    /// Gmail API用のSASL XOAUTH2文字列を生成
    pub fn generate_xoauth2_string(&self, email: &str, access_token: &str) -> String {
        let auth_string = format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token);
        general_purpose::STANDARD.encode(auth_string.as_bytes())
    }

    /// アクセストークンの有効性を検証
    pub async fn validate_token(&self, access_token: &str) -> Result<bool> {
        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v1/tokeninfo")
            .query(&[("access_token", access_token)])
            .send()
            .await
            .context("Failed to validate token")?;

        Ok(response.status().is_success())
    }
}

impl Default for GoogleOAuthConfig {
    fn default() -> Self {
        Self {
            client_id: "YOUR_GOOGLE_CLIENT_ID".to_string(),
            client_secret: "YOUR_GOOGLE_CLIENT_SECRET".to_string(),
            redirect_uri: GOOGLE_REDIRECT_URI.to_string(),
        }
    }
}

// OAuth2認証フロー管理
pub struct OAuthFlowManager {
    pending_flows: HashMap<String, CsrfToken>,
}

impl OAuthFlowManager {
    pub fn new() -> Self {
        Self {
            pending_flows: HashMap::new(),
        }
    }

    pub fn start_flow(&mut self, state: String, csrf_token: CsrfToken) {
        self.pending_flows.insert(state, csrf_token);
    }

    pub fn validate_and_complete_flow(&mut self, state: &str, received_state: &str) -> Result<()> {
        let stored_token = self
            .pending_flows
            .remove(state)
            .context("Invalid or expired OAuth flow")?;

        if stored_token.secret() != received_state {
            anyhow::bail!("CSRF token mismatch");
        }

        Ok(())
    }
}

impl Default for OAuthFlowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xoauth2_string_generation() {
        let config = GoogleOAuthConfig::default();
        let client = GoogleOAuthClient::new(config).unwrap();

        let email = "test@gmail.com";
        let access_token = "test_token";
        let xoauth2_string = client.generate_xoauth2_string(email, access_token);

        // Base64デコードして内容を確認
        let decoded = general_purpose::STANDARD.decode(&xoauth2_string).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        assert!(decoded_str.contains("user=test@gmail.com"));
        assert!(decoded_str.contains("auth=Bearer test_token"));
    }
}
