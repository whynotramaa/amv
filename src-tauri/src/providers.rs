//! Non-secret provider configuration. Credentials stay in the OS vault.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApiProvider {
    Gemini,
    Deepseek,
}

impl ApiProvider {
    pub fn credential_target(self) -> &'static str {
        match self {
            Self::Gemini => "apikey-gemini",
            Self::Deepseek => "apikey-deepseek",
        }
    }
}

pub fn http_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(concat!("Harness/", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(90))
        .build()?)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

pub async fn discover_models(
    client: &reqwest::Client,
    endpoint: Url,
    bearer: &str,
    chatgpt: bool,
) -> Result<Vec<ModelInfo>> {
    use futures_util::StreamExt;
    let request = async {
        let request = client.get(endpoint).bearer_auth(bearer);
        let response = request
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("Couldn't reach the model service"))?;
        if !response.status().is_success() {
            bail!("Model discovery failed ({})", response.status().as_u16());
        }
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| anyhow::anyhow!("Couldn't read the model catalog"))?;
            if bytes.len() + chunk.len() > 1024 * 1024 {
                bail!("Model catalog is too large");
            }
            bytes.extend_from_slice(&chunk);
        }
        parse_models(&bytes, chatgpt)
    };
    tokio::time::timeout(std::time::Duration::from_secs(10), request)
        .await
        .map_err(|_| anyhow::anyhow!("Model discovery timed out"))?
}

fn parse_models(bytes: &[u8], chatgpt: bool) -> Result<Vec<ModelInfo>> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).context("Invalid model catalog")?;
    let entries = value
        .get(if chatgpt { "models" } else { "data" })
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Unrecognized model catalog"))?;
    if entries.len() > 2048 {
        bail!("Too many models in catalog");
    }
    let mut models = Vec::new();
    for entry in entries {
        if chatgpt && entry.get("visibility").and_then(|v| v.as_str()) != Some("list") {
            continue;
        }
        let Some(id) = entry
            .get(if chatgpt { "slug" } else { "id" })
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        if id.is_empty() || id.len() > 200 || id.chars().any(char::is_control) {
            continue;
        }
        let name = entry
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or(id);
        if name.len() > 512 || name.chars().any(char::is_control) {
            continue;
        }
        if models.iter().any(|model: &ModelInfo| model.id == id) {
            continue;
        }
        models.push(ModelInfo {
            id: id.into(),
            name: name.into(),
        });
    }
    Ok(models)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub provider: ApiProvider,
    pub base_url: String,
    pub model: Option<String>,
    pub allow_fallback: bool,
}

impl ProviderConfig {
    pub fn endpoint(&self, path: &str) -> Result<Url> {
        let mut base =
            Url::parse(self.base_url.trim()).context("Enter a valid provider base URL")?;
        if base.scheme() != "https" || base.host_str().is_none() {
            bail!("Provider base URLs must use HTTPS");
        }
        if !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
        {
            bail!("Base URLs cannot contain credentials, query parameters, or fragments");
        }
        let directory = format!("{}/", base.path().trim_end_matches('/'));
        base.set_path(&directory);
        Ok(base.join(path)?)
    }

    pub fn validate(&self) -> Result<()> {
        if self.base_url.len() > 2048 {
            bail!("Provider base URL is too long");
        }
        self.endpoint("models")?;
        if let Some(model) = &self.model {
            if model.trim().is_empty() || model.len() > 200 || model.chars().any(char::is_control) {
                bail!("Choose a valid provider model");
            }
        }
        Ok(())
    }
}

pub fn defaults() -> Vec<ProviderConfig> {
    vec![
        ProviderConfig {
            provider: ApiProvider::Gemini,
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai/".into(),
            model: None,
            allow_fallback: false,
        },
        ProviderConfig {
            provider: ApiProvider::Deepseek,
            base_url: "https://api.deepseek.com/".into(),
            model: None,
            allow_fallback: false,
        },
    ]
}

pub fn validate_all(configs: &[ProviderConfig]) -> Result<()> {
    if configs.len() != 2 || configs[0].provider == configs[1].provider {
        bail!("Configure Gemini and DeepSeek once each");
    }
    for config in configs {
        config.validate()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_api_paths_and_rejects_credential_leaks() {
        let mut configs = defaults();
        assert_eq!(
            configs[0].endpoint("models").unwrap().as_str(),
            "https://generativelanguage.googleapis.com/v1beta/openai/models"
        );
        configs[1].base_url = "https://example.com/custom/v1".into();
        assert_eq!(
            configs[1].endpoint("chat/completions").unwrap().as_str(),
            "https://example.com/custom/v1/chat/completions"
        );
        for bad in [
            "http://example.com",
            "https://key@example.com",
            "https://example.com?key=secret",
            "https://example.com/#fragment",
        ] {
            configs[1].base_url = bad.into();
            assert!(validate_all(&configs).is_err(), "accepted {bad}");
        }
        assert!(validate_all(&[configs[0].clone(), configs[0].clone()]).is_err());
        assert!(defaults()
            .iter()
            .all(|p| !p.allow_fallback && p.model.is_none()));
        let models = parse_models(br#"{"models":[{"slug":"visible","visibility":"list","display_name":"Visible"},{"slug":"hidden","visibility":"hidden"}]}"#, true).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "visible");
        assert_eq!(
            parse_models(br#"{"data":[{"id":"api-model"}]}"#, false).unwrap()[0].name,
            "api-model"
        );
    }
}
