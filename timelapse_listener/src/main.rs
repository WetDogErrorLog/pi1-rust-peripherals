use axum::{
    extract::{Json, State, DefaultBodyLimit},
    routing::post,
    http::StatusCode,
    response::IntoResponse,
    Router,
};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::sync::Arc;
use image_handling::{CameraPacket, handle_image_post};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct AppState{
    storage_dir_path: String,
    service_port: u16,
}

#[tokio::main]
async fn main() {
    let listener_str = std::fs::read_to_string("./listener_config.json")
        .expect("failed to read config to string");
    let listener_config: AppState = serde_json::from_str(&listener_str)
        .expect("failed to unpack the json");
    let storage_dir = listener_config.storage_dir_path.clone();     
    let service_port = listener_config.service_port.clone();  

    let shared_state = Arc::new(listener_config);
    
    // TODO: When we swap away from YUYV we can reduce body limit to default.
    let app = Router::new()
        .route("/upload_image", post(upload_handler))
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0,0,0,0], service_port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening..."); 
    axum::serve(listener, app).await.unwrap();
}


async fn upload_handler(
    State(state): State<Arc<AppState>>,
    Json(packet): Json<CameraPacket>,
) -> impl IntoResponse {
    println!("Recieved image {}, {}", packet.project_folder, packet.file_name_root);
    match handle_image_post(packet, &state.storage_dir_path){
        Ok(_) => {
            println!("saved");
            (StatusCode::OK, "Save success")
        }
        Err(e) => {
            eprintln!("Save error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save png")
        }
    }
}
