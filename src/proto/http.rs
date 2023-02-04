use axum::http::HeaderMap;
use http_body::Body;
use bytes::Bytes;
use crate::router;

use std::{
    task::Context,
    task::Poll,
    pin::Pin,
};

pub struct Stream {
    reader: router::Reader,
}

impl Stream {
    pub fn new(reader: router::Reader) -> Self {
        Self {
            reader,
        }
    }
}

impl Body for Stream {
    type Data = Bytes;
    type Error = anyhow::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.as_mut().reader.poll_read(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => {
                Poll::Ready(res.and_then(|b| Some(Ok(Bytes::from(b)))))
            },
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
