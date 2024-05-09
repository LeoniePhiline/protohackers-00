#![allow(unreachable_code)]

use std::{fmt::Debug, time::Duration};

use color_eyre::eyre::{self, bail, Result, WrapErr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{debug, error, info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_filter(EnvFilter::builder().from_env()?),
        )
        .try_init()
        .wrap_err("failed initializing tracing")?;

    info!("{:=^40}", " Protohackers 00: Smoke test ");

    // Listen on a port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    let mut handles = Vec::new();

    // Accept connections
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => { break; }
            res = listener.accept() => {
                let (stream, _) = res.wrap_err("accept failure")?;

                info!("Accepted connection.");

                let (read_half, write_half) = stream.into_split();

                // Spawn a new task for this connection.
                // All tasks are executed by the runtime concurrently.
                handles.push(tokio::spawn(
                    // Give the connection 20 seconds to do its business
                    tokio::time::timeout(
                        Duration::from_secs(20),
                        // Read from and write to the connection.
                        async move {
                            handle_connection(read_half, write_half)
                                .await
                                .inspect_err(|err| error!("Handling the connection failed: {err:?}"))
                        },
                    ),
                ));

            }
            else => break
        };
    }

    for handle in handles {
        let Ok(no_toilet) = {
            handle
                .await
                .wrap_err("join failure")?
                .wrap_err("timeout")?
                .wrap_err("connection processing error")
        }
        .inspect_err(|err| error!("{err:?}")) else {
            continue;
        };

        debug!("{no_toilet:*^80?}");
    }

    info!("shutdown");

    Ok(())
}

#[instrument]
async fn handle_connection2<R, W>(mut reader: R, mut writer: W) -> Result<u64>
where
    R: AsyncRead + Unpin + Debug,
    W: AsyncWrite + Unpin + Debug,
{
    tokio::io::copy(&mut reader, &mut writer)
        .await
        .wrap_err("copy error")
}

#[instrument(skip(reader, writer))]
async fn handle_connection<R, W>(mut reader: R, mut writer: W) -> Result<()>
where
    R: AsyncRead + Unpin + Debug,
    W: AsyncWrite + Unpin + Debug,
{
    info!("Handling connection.");
    // panic!("oh no");

    let mut buf = [0u8; 1024 * 8];

    // Echo incoming reads on this connection
    #[allow(clippy::never_loop)]
    loop {
        debug!("Reading bytes...");

        let num_bytes_read = reader.read(&mut buf).await?;

        info!(
            they_said = ?String::from_utf8_lossy(&buf[..num_bytes_read])
        );

        debug!(%num_bytes_read);

        // End this connection if we read nothing (client hung up)
        if num_bytes_read == 0 {
            debug!("Client hung up.");
            break;
        }

        #[cfg(feature = "she-said-he-said")]
        writer.write_all(b"you said: ").await?;

        writer.write_all(&buf[..num_bytes_read]).await?;

        #[cfg(feature = "she-said-he-said")]
        writer.write_all(b"\r\n").await?;
    }

    println!("finished connection");

    eyre::Ok(())
}
