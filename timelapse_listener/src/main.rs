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
        storage_dir: "/home/shared_space/timelapse_data".to_string(),
    });
    
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
    println!("Recieved image {}, {}", packet.project_folder, packet.file_name_root);
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
