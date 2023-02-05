<!--lint disable no-literal-urls-->
<div align="center">
  <h1>MediaServer-RS</h1>
</div>
<br/>
<div align="center">
  <strong>Media Server implemented by ❤️ Rust</strong>
</div>
<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/mycrl/media-server-rs/cargo-test.yml?branch=main"/>
  <img src="https://img.shields.io/github/license/mycrl/media-server-rs"/>
  <img src="https://img.shields.io/github/issues/mycrl/media-server-rs"/>
  <img src="https://img.shields.io/github/stars/mycrl/media-server-rs"/>
</div>
<br/>
<br/>


A rust-implemented media server, the project does not introduce complex features, but simply remuxing media transfer protocols and packaging containers.


### What stage is the project at now?

Only supports rtmp push to websocket/http flv, currently this project is in the early development stage, and only implements the basic media server framework.


### next?

In order to make this project sustainable and reduce complexity, I will introduce ffmpeg to help me process the conversion of media container packages, or if there is a need for subsequent audio and video encoding conversion, I will also use ffmpeg for processing.

I hope this project can support rtmp, http flv, websocket flv, rtsp, hls, dash, http mpegts. (cautious support for webrtc)


### Do you want to try?

```bash
cargo run --release -- --config=./media-server.toml
```

Regarding the configuration file, there is currently no more descriptive information;

```toml
[proto.rtmp]
listen = "127.0.0.1:1935"
band_width = 5000000

[proto.websocket_flv]
listen = "127.0.0.1:8080"
max_send_queue = 5
max_message_size = 50000
max_frame_size = 5000
accept_unmasked_frames = true

[proto.http_flv]
listen = "127.0.0.1:8081"
allow_origin = "*"

[log]
level = "info"
```


## License

[GPL](./LICENSE)
Copyright (c) 2022 Mr.Panda.
