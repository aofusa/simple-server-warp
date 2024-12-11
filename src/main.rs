use clap::Parser;
use tracing_subscriber;
use warp::Filter;
use nix::sys::resource::Resource;
use tokio::net::TcpSocket;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, help = "接続を受けるワーカースレッド数。デフォルトはCPUコア数。")]
    r#worker_thread: Option<usize>,

    #[arg(long, help = "ワーカースレッドごとの最大接続可能数。デフォルトは1024。推奨値はファイルディスクリプタのソフトリミット数÷ワーカースレッド数÷4")]
    r#worker_connection: Option<u32>,

    #[arg(long, help = "ファイルディスクリプタ数の上限。デフォルトはOS設定のソフトリミット数。推奨値はソフトリミット数÷ワーカースレッド数。")]
    r#worker_rlimit_nofile: Option<u64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());
    tracing_subscriber::fmt()
      .with_env_filter(filter)
      .init();

    let arg = Args::parse();

    if let Some(rlimit) = arg.worker_rlimit_nofile {
        let (slimit, hlimit) = nix::sys::resource::getrlimit(Resource::RLIMIT_NOFILE)?;
        nix::sys::resource::setrlimit(
            Resource::RLIMIT_NOFILE,
            rlimit,
            hlimit
        )?;
        println!("change rlimit from {} to {}.", slimit, rlimit);
    }

    let runtime = {
        let mut rt = if arg.worker_thread != Some(1) {
            &mut tokio::runtime::Builder::new_multi_thread()
        } else {
            &mut tokio::runtime::Builder::new_current_thread()
        };

        if let Some(threads) = arg.worker_thread { rt = rt.worker_threads(threads) }

        rt
          .enable_all()
          .build()?
    };

    // GET /
    let hello_world = warp::path::end()
      .and(warp::get())
      .map(|| "Hello, World at root!");

    // GET /hi
    let hi = warp::path("hi")
      .and(warp::get())
      .map(|| "Hello, World!");

    // POST /echo
    let echo = warp::path("echo")
      .and(warp::post())
      .and(warp::body::json())
      .map(|body: serde_json::Value| warp::reply::json(&body));

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
      .and(warp::get())
      .map(|name| format!("Hello, {}!", name));

    // GET /dir => map to directory "./"
    let file = warp::path("dir")
      .and(warp::get())
      .and(warp::fs::dir("./"));

    let routes = hello_world
      .or(hi)
      .or(hello)
      .or(file)
      .or(echo);

    let non_tls_server = warp::serve(routes.clone().with(warp::trace::request()))
      .run(([0, 0, 0, 0], 3030));

    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    socket.set_reuseport(true)?;
    socket.bind(([0, 0, 0, 0], 3031).into())?;

    let tls_server = warp::serve(routes.with(warp::trace::request()))
      .tls()
      .cert_path("./credential/server.crt")
      .key_path("./credential/server.key")
      .run(([0, 0, 0, 0], 3031));

    let _guard = runtime.enter();
    runtime.block_on(async {
        // non tls server
        let handle = runtime.spawn(non_tls_server);

        if let Some(connections) = arg.worker_connection {
            let _listener = socket.listen(connections).unwrap();
            // TODO: warpのfilterに接続する
        }

        // use tls
        tls_server.await;

        handle.abort();
    });

    Ok(())
}
