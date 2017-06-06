# mrsc

Multi Requester Single Consumer

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
