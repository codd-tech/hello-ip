use axum::{
    extract::ConnectInfo,
    routing::get,
    Router,
    http::{HeaderMap, StatusCode},
};
use std::net::SocketAddr;
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hello_ip=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let app = Router::new()
        .route("/", get(default_handler))
        .route("/livez", get(livez_handler));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");


    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn default_handler(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> String {
    let client_ip = extract_client_ip(&headers, addr.ip());
    info!("Request from {}, resolved IP: {}", addr.ip(), client_ip);
    client_ip.to_string()
}

async fn livez_handler() -> String {
    "Healthy".to_string()
}

fn extract_client_ip(headers: &HeaderMap, fallback_ip: std::net::IpAddr) -> std::net::IpAddr {
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            if let Some(first_ip) = xff_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return ip;
                }
            }
        }
    }
    
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(xri_str) = xri.to_str() {
            if let Ok(ip) = xri_str.parse() {
                return ip;
            }
        }
    }
    
    if let Some(xci) = headers.get("x-client-ip") {
        if let Ok(xci_str) = xci.to_str() {
            if let Ok(ip) = xci_str.parse() {
                return ip;
            }
        }
    }
    
    if let Some(forwarded) = headers.get("forwarded") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            for part in forwarded_str.split(';') {
                let part = part.trim();
                if part.starts_with("for=") {
                    let ip_part = &part[4..];
                    let ip_str = ip_part
                        .trim_matches('"')
                        .split(':')
                        .next()
                        .unwrap_or(ip_part)
                        .trim_matches('[')
                        .trim_matches(']');
                    
                    if let Ok(ip) = ip_str.parse() {
                        return ip;
                    }
                }
            }
        }
    }
    
    fallback_ip
}