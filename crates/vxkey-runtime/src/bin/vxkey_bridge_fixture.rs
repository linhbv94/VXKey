//! Two-process native bridge fixture. No IMK registration or keyboard synthesis.
#[cfg(target_os = "macos")]
fn main() {
    use serde_json::{Value, json};
    use std::{
        io::{self, BufRead, Write},
        sync::{Arc, Mutex},
        time::Duration,
    };
    use vxkey_runtime::{
        bridge::{
            self, Control, NativeStatus,
            local::{Client, Host},
        },
        config::Platform,
        service::Service,
        storage::ConfigStore,
    };
    if std::env::var("VXKEY_BRIDGE_DEV_TRUST").as_deref() != Ok("1") {
        std::process::exit(2);
    }
    let root = std::path::PathBuf::from(std::env::var_os("VXKEY_DATA_DIR").expect("fixture root"));
    let role = std::env::args().nth(1).expect("host/client");
    if role == "host" {
        let store = ConfigStore::open(root.join("config.json"), Platform::Macos).unwrap();
        store.save_initial().unwrap();
        let service = Arc::new(Mutex::new(Service::new(store)));
        let host = Host::start(&root, service.clone()).unwrap();
        println!("{}", json!({"ready":true}));
        io::stdout().flush().unwrap();
        for line in io::stdin().lock().lines() {
            let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
            let result = match request["fixture"].as_str() {
                Some("control") => {
                    let c: Control = serde_json::from_value(request["control"].clone()).unwrap();
                    match host.request(c) {
                        Ok(s) => json!({"ok":true,"status":s}),
                        Err(e) => json!({"ok":false,"error":e}),
                    }
                }
                Some("latch") => json!({"latched":host.latch.is_set()}),
                _ => service.lock().unwrap().dispatch(request),
            };
            println!("{result}");
            io::stdout().flush().unwrap();
        }
    } else if role == "client" {
        let client = Client::start(&root).unwrap();
        println!("{}", json!({"ready":true}));
        io::stdout().flush().unwrap();
        for line in io::stdin().lock().lines() {
            let v: Value = serde_json::from_str(&line.unwrap()).unwrap();
            if let Some(ms) = v["stall_ms"].as_u64() {
                std::thread::sleep(Duration::from_millis(ms.min(8000)));
                println!("{{}}");
                continue;
            }
            let status: NativeStatus = serde_json::from_value(v).unwrap();
            match client.tick(status) {
                Ok(d) => println!("{}", serde_json::to_string(&d).unwrap()),
                Err(e) => println!("{}", json!({"error":e})),
            };
            io::stdout().flush().unwrap();
        }
    } else {
        panic!("unknown fixture role");
    }
    let _ = bridge::FRAME_LIMIT;
}
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("macOS fixture only");
    std::process::exit(2);
}
