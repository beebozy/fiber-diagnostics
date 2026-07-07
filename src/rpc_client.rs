use serde_json::json;

pub struct FiberRpcClient {
    http: reqwest::Client,
    url: String,
}

impl FiberRpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self { http: reqwest::Client::new(), url: url.into() }
    }

    async fn call(&self, method: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let resp = self.http
            .post(&self.url)
            .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }))
            .send()
            .await?;
        let body: serde_json::Value = resp.json().await?;
        if let Some(err) = body.get("error") {
            anyhow::bail!("RPC error calling {method}: {err}");
        }
        Ok(body["result"].clone())
    }

    pub async fn node_info(&self) -> anyhow::Result<serde_json::Value> {
        self.call("node_info", json!([])).await
    }

    pub async fn list_peers(&self) -> anyhow::Result<serde_json::Value> {
        self.call("list_peers", json!([])).await
    }

    pub async fn list_channels(&self) -> anyhow::Result<serde_json::Value> {
        self.call("list_channels", json!([{}])).await
    }

    pub async fn graph_nodes(&self) -> anyhow::Result<serde_json::Value> {
    self.call("graph_nodes", json!([{}])).await
}

pub async fn graph_channels(&self) -> anyhow::Result<serde_json::Value> {
    self.call("graph_channels", json!([{}])).await
}
}