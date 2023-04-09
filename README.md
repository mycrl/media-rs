<!--lint disable no-literal-urls-->
<div align="center">
  <h1>media-rs</h1>
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


## License

[GPL](./LICENSE)
Copyright (c) 2022 Mr.Panda.
