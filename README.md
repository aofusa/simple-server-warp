Simple server using Warp framework
=====


Warpを使ったwebサーバサンプル


[Warp](https://github.com/seanmonstar/warp)  


http1,http2,tlsに対応

起動後はhttp,httpsがそれぞれ起動する


使い方
-----

```sh
Usage: simple-server-warp [OPTIONS]

Options:
      --worker-thread <WORKER_THREAD>
          接続を受けるワーカースレッド数。デフォルトはCPUコア数。
      --worker-connection <WORKER_CONNECTION>
          ワーカースレッドごとの最大接続可能数。デフォルトは1024。推奨値はファイルディスクリプタのソフトリミット数÷ワーカースレッド数÷4
      --worker-rlimit-nofile <WORKER_RLIMIT_NOFILE>
          ファイルディスクリプタ数の上限。デフォルトはOS設定のソフトリミット数。推奨値はソフトリミット数÷ワーカースレッド数。
  -h, --help
          Print help
  -V, --version
          Print version
```

例
```sh
# デフォルト設定で起動
cargo run

# シングルスレッドで起動
cargo run -- --worker-thread 1

# 4スレッドで起動, ログ出力をinfo以上に設定
RUST_LOG=info cargo run -- --worker-thread 4
```

起動後は http://localhost:3030, https://localhost:3031/ でアクセス可能

- GET / -> "Hello, World at root!"
- GET /hi -> "Hello, World!"
- GET /hello/:name -> "Hello, <name>!"
- GET /dir/:filename -> 実行ディレクトリ内のフォルダやファイルを表示

