use crate::router::RouterReceiver;

use std::{pin::Pin, task::Context, task::Poll};

use axum::http::HeaderMap;
use bytes::Bytes;
use http_body::Body;

pub struct Stream {
    receiver: RouterReceiver,
}

impl Stream {
    pub fn new(receiver: RouterReceiver) -> Self {
        Self { receiver }
    }
}

impl Body for Stream {
    type Data = Bytes;
    type Error = anyhow::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.as_mut().receiver.poll_read(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => Poll::Ready(res.and_then(|b| Some(Ok(Bytes::from(b))))),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
