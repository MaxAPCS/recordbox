# RecordBox
RecordBox is a mutable music library manager. It's designed to fill the gap with subsonic-type music players which never modify the underlying files and allow for the library to be tagged, standardized, and organized more effectively. It's built completely in Rust and supports statically-linked standalone compilation, so it can be dropped into any environment.

Currently, the frontend proof-of-concept is in an incomplete state. It cannot interact with most endpoints of the backend, but the framework is in place for the missing features. To get a full sense of the backend's capabilities, make requests to the endpoints defined in [server.rs](backend/src/server.rs).

## Building
This project uses the cargo package manager. The frontend must be built for `wasm32-unknown-unknown`, and the backend is designed to be built for `x86_64-unknown-linux-musl`. A working build chain can be reproduced from [ci.yml](.github/workflows/ci.yml). For development, a [script](run.sh) is provided to run an unoptimized version.

You can download the latest linux release from the CI pipeline at https://nightly.link/MaxAPCS/recordbox/workflows/ci/main/artifact.zip.

## Running
Configuration is required for the server, and an example is provided in [config.example.yaml](config.example.yaml). Rename it to `config.yaml`, change the relevant options, and create the directory specified in `library:`. Environment variables are also parsed as config options, as specified in the comments.

## Bibliography
### Frameworks
- axum: Web Framework
- mp4ameta: Metadata Parser & Writer
- image: Image Loading
- wgpu: GPU Interface

### Generative AI Use Disclosure
Generative AI assistance was used solely to locate relevant portions of code, and to fix specific bugs. The program architecture and most specifics were entirely written by hand.

