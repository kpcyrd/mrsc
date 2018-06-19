# mrsc [![Build Status](https://travis-ci.org/kpcyrd/mrsc.svg?branch=master)](https://travis-ci.org/kpcyrd/mrsc) [![Crates.io](https://img.shields.io/crates/v/mrsc.svg)](https://crates.io/crates/mrsc)

mpsc with requests. This is a basic building block based on rusts mpsc if you have multiple workers that need to execute transactions on a shared state, without having to share your state struct across all threads. Beware that transactions are blocking, so it's recommended to avoid expensive code in the transaction handler.

## Installation

Add this to your `Cargo.toml` dependency list:
```toml,ignore
[dependencies]
mrsc = "0.3"
```

Add this to your crate root:
```rust,ignore
extern crate mrsc;
```

## Example

```rust
use mrsc;
use std::thread;

let server: mrsc::Server<u32, String> = mrsc::Server::new();
let channel = server.pop();

thread::spawn(move || {
    let req = server.recv().unwrap();
    let reply = {
        let msg = req.get();
        println!("request: {:?}", msg);

        "hello world".to_string()
    };
    req.reply(reply).unwrap();
});

let response = channel.req(123).unwrap();
let reply = response.recv().unwrap();
println!("response: {:?}", reply);
```

## License

MIT
