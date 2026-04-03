use dotenvy::dotenv;
use std::env;

fn main() {
    let _ = dotenv(); 

    println!("cargo:rerun-if-changed=.env");

    if let Ok(client_id) = env::var("IGDB_CLIENT_ID") 
    {
        println!("cargo:rustc-env=IGDB_CLIENT_ID={}", client_id);
    }
    
    if let Ok(client_secret) = env::var("IGDB_CLIENT_SECRET") 
    {
        println!("cargo:rustc-env=IGDB_CLIENT_SECRET={}", client_secret);
    }

    tauri_build::build()    
}