use clap::Parser;
use hyper::body::Body;
use hyper::client::Client;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Request, Response, Server, Uri};
use log::{error, info};
use std::convert::Infallible;
use std::net::SocketAddr;

// 命令行参数结构
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // 目标服务器地址（例如：example.com:80）
    #[arg(short, long, required = true)]
    target: String,
}

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args = Args::parse();
    let target_server = args.target;
    
    // 初始化日志系统
    env_logger::init();
    
    // 配置代理服务器
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    
    info!("Starting proxy server on {}", addr);
    info!("Target server: {}", target_server);
    
    // 创建HTTP客户端
    let client = Client::new();
    
    // 创建服务
    let make_svc = make_service_fn(move |_conn| {
        let client = client.clone();
        let target_server = target_server.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let client = client.clone();
                let target_server = target_server.clone();
                async move {
                    proxy_handler(client, req, &target_server).await
                }
            }))
        }
    });
    
    // 启动服务器
    let server = Server::bind(&addr).serve(make_svc);
    
    // 运行服务器
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
}

async fn proxy_handler(client: Client<hyper::client::HttpConnector>, req: Request<Body>, target_server: &str) -> Result<Response<Body>, Infallible> {
    info!("Received request: {} {}/{}", req.method(), target_server, req.uri().path_and_query().map(|p| p.as_str()).unwrap_or(""));
    
    // 构建目标URI
    let target_uri = match build_target_uri(target_server, req.uri()) {
        Ok(uri) => uri,
        Err(e) => {
            error!("Error building target URI: {}", e);
            return Ok(build_error_response(400, &format!("Bad Request: {}", e)));
        }
    };
    
    info!("Forwarding request to: {}", target_uri);
    
    // 创建一个新的请求，转发到目标服务器
    let mut builder = Request::builder()
        .method(req.method())
        .uri(target_uri);
    
    // 复制请求头
    for (key, value) in req.headers() {
        if key != "host" {
            builder = builder.header(key, value);
        }
    }
    
    // 添加host头
    builder = builder.header("host", target_server);
    
    // 构建请求
    let request = match builder.body(req.into_body()) {
        Ok(req) => req,
        Err(e) => {
            error!("Error building request: {}", e);
            return Ok(build_error_response(500, &format!("Internal Server Error: {}", e)));
        }
    };
    
    // 发送请求到目标服务器并获取响应
    match client.request(request).await {
        Ok(response) => {
            info!("Received response: {}", response.status());
            Ok(response)
        },
        Err(e) => {
            error!("Error forwarding request: {}", e);
            Ok(build_error_response(502, &format!("Bad Gateway: {}", e)))
        },
    }
}

fn build_target_uri(target_server: &str, req_uri: &Uri) -> Result<Uri, String> {
    // 构建完整目标URI
    let target = match req_uri.path_and_query() {
        Some(path_and_query) => format!("http://{}{}", target_server, path_and_query.as_str()),
        None => format!("http://{}", target_server),
    };
    
    // 解析为URI类型并返回
    target.parse::<Uri>().map_err(|e| e.to_string())
}

fn build_error_response(status: u16, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message.to_string()))
        .unwrap()
}
