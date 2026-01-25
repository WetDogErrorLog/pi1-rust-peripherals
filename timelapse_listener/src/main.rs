use axum::{
    extract::{Json, State, DefaultBodyLimit},
    routing::post,
    http::StatusCode,
    response::IntoResponse,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use image_handling::{CameraPacket, handle_image_post};


// TODO: Replace with reading from local config.
struct AppState{
    storage_dir: String,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        storage_dir: "/home/shared_space/timlapse_data".to_string(),
    });

    std::fs::create_dir_all(&shared_state.storage_dir).unwrap();

    // TODO: When we swap away from YUYV we can reduce body limit to default.
    let app = Router::new()
        .route("/upload_image", post(upload_handler))
        .layer(DefaultBodyLimit::max(15 * 1024 * 1024))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0,0,0,0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening..."); 
    axum::serve(listener, app).await.unwrap();
}


async fn upload_handler(
    State(state): State<Arc<AppState>>,
    Json(packet): Json<CameraPacket>,
) -> impl IntoResponse {
    // TODO: since the json is deserialized automatically,
    // the method can bechanged to take the packet object.
    println!("Recieved image {}", packet.file_name);
    // let binary_packet = bincode::serialize(&packet).unwrap();
    match handle_image_post(packet, &state.storage_dir){
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
