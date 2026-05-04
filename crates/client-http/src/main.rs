mod api;
mod http;
mod wss;

fn main() -> std::io::Result<()> {
    http::run_server()
}
