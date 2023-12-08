use ahash::AHashMap;
use anyhow::{anyhow, Result};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{handshake::server::*, protocol::WebSocketConfig};
use tokio_tungstenite::{accept_hdr_async_with_config, WebSocketStream};

#[derive(Debug, Clone)]
pub struct Query {
    pub name: String,
    pub key: String,
}

impl Query {
    pub fn from_str(src: &str) -> Result<Self> {
        let querys = parse(src);
        Ok(Self {
            name: querys
                .get("name")
                .ok_or_else(|| anyhow!("name is not found!"))?
                .to_string(),
            key: querys
                .get("key")
                .ok_or_else(|| anyhow!("key is not found!"))?
                .to_string(),
        })
    }
}

pub fn parse(src: &str) -> AHashMap<String, String> {
    let mut querys = AHashMap::with_capacity(5);
    src.split('&').for_each(|item| {
        if let Some((k, v)) = item.split_once('=') {
            querys.insert(k.to_string(), v.to_string());
        }
    });

    querys
}

struct Guard<'a> {
    query: &'a mut String,
}

impl Callback for Guard<'_> {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        if let Some(query) = request.uri().query() {
            self.query.push_str(query);
        }

        Ok(response)
    }
}

pub async fn accept(
    socket: TcpStream,
    cfg: Option<WebSocketConfig>,
) -> Result<(WebSocketStream<TcpStream>, Query)> {
    let mut query = String::new();
    let stream = accept_hdr_async_with_config(socket, Guard { query: &mut query }, cfg).await?;
    let query = Query::from_str(&query)?;
    Ok((stream, query))
}
