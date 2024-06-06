fn main() {}

// const AP_SSID: &str = "esp32c3";
// const AP_PASSWORD: &str = "12345678";

// fn main() -> anyhow::Result<()> {
//     esp_idf_svc::sys::link_patches();
//     esp_idf_svc::log::EspLogger::initialize_default();

//     // `async-io` uses the ESP IDF `eventfd` syscall to implement async IO.
//     // If you use `tokio`, you still have to do the same as it also uses the `eventfd` syscall
//     esp_idf_svc::io::vfs::initialize_eventfd(5).unwrap();

//     // This thread is necessary because the ESP IDF main task thread is running with a very low priority that cannot be raised
//     // (lower than the hidden posix thread in `async-io`)
//     // As a result, the main thread is constantly starving because of the higher prio `async-io` thread
//     //
//     // To use async networking IO, make your `main()` minimal by just spawning all work in a new thread
//     std::thread::Builder::new()
//         .stack_size(60000)
//         .spawn(run)
//         .unwrap()
//         .join()
//         .unwrap()
//         .unwrap();

//     Ok(())
// }

// fn run() -> Result<(), anyhow::Error> {
//     // Any executor would do. We just use the local executor from the `futures` crate
//     // As for why we need an executor - just for a simple way to spawn the accepted connections
//     // in the `tcp_server` server
//     let mut local_executor = LocalPool::new();
//     let spawner = local_executor.spawner();

//     local_executor.spawner().spawn_local(
//         async move {
//             // Keep it around or else the wifi will stop
//             let _wifi = wifi_create().await?;

//             tcp_client().await?;
//             tcp_server(spawner).await?;

//             Result::<_, anyhow::Error>::Ok(())
//         }
//         .map(Result::unwrap),
//     )?;

//     local_executor.run();

//     Ok(())
// }
