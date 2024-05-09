#![allow(unreachable_code)]

use std::time::Duration;

use color_eyre::eyre::{self, Result, WrapErr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("{:-^80}", " Protohackers 00: Smoke test ");

    // Listen on a port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    // let mut handles = Vec::new();

    // Accept connections
    #[allow(clippy::never_loop)]
    loop {
        let (mut stream, _addr) = listener.accept().await?;

        // Spawn a new task for this connection.
        // All tasks are executed by the runtime concurrently.
        tokio::spawn(
            // Give the connection 20 seconds to do its business
            tokio::time::timeout(
                Duration::from_secs(20),
                // Do the business
                async move {
                    panic!("oh no");

                    let mut buf = [0u8; 1024];

                    let (mut read_half, mut write_half) = stream.split();

                    // Echo incoming reads on this connection
                    loop {
                        let num_bytes_read = read_half.read(&mut buf).await?;

                        // tokio::time::sleep(Duration::from_secs(21)).await;

                        // End this connection if we read nothing (client hung up)
                        if num_bytes_read == 0 {
                            break;
                        }

                        write_half.write_all(&buf[..num_bytes_read]).await?;
                    }

                    eyre::Ok(())
                },
            ),
        );
    }

    // for handle in handles {
    //     // Whaaat
    //     handle
    //         .await
    //         .wrap_err("join failure")?
    //         .wrap_err("timeout")?
    //         .wrap_err("connection processing error")?;
    // }

    Ok(())
}
