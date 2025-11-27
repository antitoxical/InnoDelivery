use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use tracing::Level;

#[tokio::test]
async fn test_error_coverage_bad_body() {
    dotenvy::dotenv().ok();

    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_test_writer()
        .try_init();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get addr");
    let user_service_url = format!("http://{}", addr);

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);

            let response = b"HTTP/1.1 500 Internal Server Error\r\n\
                             Content-Type: text/plain\r\n\
                             Content-Length: 100\r\n\
                             \r\n\
                             Short";
            let _ = stream.write(response);
        }
    });

    let result =
        order_service::services::order_service::try_assign_courier(&user_service_url).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();

    assert!(err_msg.contains("Failed to get response body"));
}
