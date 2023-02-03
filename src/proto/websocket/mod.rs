mod query;

use anyhow::Result;
pub use query::Query;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_hdr_async_with_config,
    WebSocketStream,
};

use tokio_tungstenite::tungstenite::{
    protocol::WebSocketConfig,
    handshake::server::*,
};

struct Guard<'a> {
    query: &'a mut String,
}

impl Callback for Guard<'_> {
    fn on_request(
        self,
        request: &Request,
        response: Response,
    ) -> Result<Response, ErrorResponse> {
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
    let stream = accept_hdr_async_with_config(
        socket,
        Guard {
            query: &mut query,
        },
        cfg,
    )
    .await?;

    let query = Query::from_str(&query)?;
    Ok((stream, query))
}
