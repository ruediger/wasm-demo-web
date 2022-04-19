Just some playing around with canvas, wasm/web_sys, websockets, and warp.

- `server/` implements an http and websocket server using warp.  Based
  off of
  https://blog.logrocket.com/how-to-build-a-websocket-server-with-rust/.
- The main directory contains a wasm project that plays around with
  canvas and sets up a simple websocket chat.

Based off of https://github.com/rustwasm/wasm-bindgen examples.

Image source https://commons.wikimedia.org/wiki/File:Mistletoebird_-_Round_Hill_Nature_Reserve.jpg License cc-by-sa 4.0 author JJ Harrison.

# Build & Run

Build the wasm module using `wasm-pack build --target web` from the
top level directory. Run the server by running `cargo run` from
`server/src`.  Unfortunately due to relative paths it has to be from
there :). Visit http://127.0.0.1:3030/ to play around.

# License

This project is licensed under either of

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.
