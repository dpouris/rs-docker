use anyhow::{self, Context};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct AuthParams<'a> {
    pub service: &'static str,
    pub client_id: &'static str,
    pub access_type: &'static str,
    pub scope: &'a str,
    pub grant_type: Option<&'static str>,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct AuthResponse {
    pub access_token: String,
    pub expires_in: u16,
    pub issued_at: String,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub token: Option<String>,
}
pub(super) enum ApiV2Status {
    Supported,
    NotSupport,
    Unauthorized(Option<String>),
}

impl<'a> AuthParams<'a> {
    pub(super) fn new(
        username: Option<&'a str>,
        password: Option<&'a str>,
        scope: &'a str,
    ) -> Self {
        let grant_type = if username.is_some() {
            Some("password")
        } else {
            None
        };
        Self {
            service: "registry.docker.io",
            client_id: "dockerengine",
            access_type: "offline",
            scope,
            grant_type,
            username,
            password,
        }
    }
}

impl<'a> TryInto<String> for &AuthParams<'a> {
    type Error = anyhow::Error;
    fn try_into(self) -> std::result::Result<String, Self::Error> {
        let auth = serde_json::to_value(self).context("tried to serialize auth")?;
        let auth = auth
            .as_object()
            .expect("AuthParams can be converted to object");
        let mut query = Vec::with_capacity(auth.len());
        for (key, val) in auth {
            match val.as_str() {
                Some(val) => query.push(format!("{key}={val}")),
                None => continue,
            }
        }
        Ok(query.join("&"))
    }
}
