//! Developer JSON-lines bridge. Explicit isolated data directory is mandatory.
use std::{
    io::{self, BufRead, Read, Write},
    path::PathBuf,
};
use vxkey_runtime::{config::Platform, service::Service, storage::ConfigStore};
fn main() {
    if let Err(e) = run() {
        eprintln!("vxkey_backend: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "--data-dir" {
        return Err("usage: vxkey_backend --data-dir <isolated directory>".into());
    }
    let path = PathBuf::from(&args[2]).join("config.json");
    let store = ConfigStore::open(path, Platform::current())?;
    store.save_initial()?;
    let mut service = Service::new(store);
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut frame = Vec::new();
        let size = input
            .by_ref()
            .take(1024 * 1024 + 1)
            .read_until(b'\n', &mut frame)
            .map_err(|_| "input_error")?;
        if size == 0 {
            break;
        }
        if frame.len() > 1024 * 1024 {
            return Err("request_too_large".into());
        }
        let value = match serde_json::from_slice::<serde_json::Value>(&frame) {
            Ok(request) => service.dispatch(request.get("request").cloned().unwrap_or(request)),
            Err(_) => {
                serde_json::json!({"ok":false,"request_id":null,"data":null,"error":{"code":"invalid_json","message_key":"errors.invalid_json","retryable":false}})
            }
        };
        serde_json::to_writer(&mut output, &value).map_err(|_| "output_error")?;
        output
            .write_all(b"\n")
            .and_then(|_| output.flush())
            .map_err(|_| "output_error")?;
    }
    Ok(())
}
