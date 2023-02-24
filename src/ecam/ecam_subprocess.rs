use crate::prelude::*;

use async_stream::stream;
use futures::TryFutureExt;
use std::process::Stdio;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::ChildStdin,
    sync::Mutex,
};
use tokio_stream::wrappers::LinesStream;

use crate::{
    ecam::{AsyncFuture, EcamDriver, EcamDriverOutput, EcamError, EcamPacketReceiver},
    protocol::*,
};

use super::EcamId;

pub struct EcamSubprocess {
    stdin: Arc<Mutex<ChildStdin>>,
    receiver: EcamPacketReceiver,
    alive: Arc<Mutex<bool>>,
}

impl EcamSubprocess {
    async fn write_stdin(&self, data: EcamDriverPacket) -> Result<(), EcamError> {
        let s = data.stringify();
        self.stdin
            .lock()
            .await
            .write(format!("S: {}\n", s).as_bytes())
            .map_ok(|_| ())
            .await?;
        Ok(())
    }

    async fn is_alive(&self) -> Result<bool, EcamError> {
        Ok(*self.alive.lock().await)
    }
}

impl EcamDriver for EcamSubprocess {
    fn read<'a>(&self) -> AsyncFuture<Option<EcamDriverOutput>> {
        Box::pin(self.receiver.recv())
    }

    fn write<'a>(&self, data: EcamDriverPacket) -> AsyncFuture<()> {
        Box::pin(self.write_stdin(data))
    }

    fn alive(&self) -> AsyncFuture<bool> {
        Box::pin(self.is_alive())
    }

    fn scan<'a>() -> AsyncFuture<'a, (String, EcamId)>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

pub async fn stream(
    mut child: tokio::process::Child,
    alive: Arc<Mutex<bool>>,
) -> Result<impl StreamExt<Item = EcamDriverOutput>, EcamError> {
    let mut stderr =
        LinesStream::new(BufReader::new(child.stderr.take().expect("stderr was missing")).lines());
    let mut stdout =
        LinesStream::new(BufReader::new(child.stdout.take().expect("stdout was missing")).lines());

    let stdout = stream! {
        while let Some(Ok(s)) = stdout.next().await {
            if s == "R: READY" {
                yield EcamDriverOutput::Ready;
            } else if let Some(s) = s.strip_prefix("R: ") {
                if let Ok(bytes) = hex::decode(&s) {
                    yield EcamDriverOutput::Packet(EcamDriverPacket::from_vec(bytes));
                } else {
                    trace_packet!("Failed to decode '{}'", s);
                }
            } else {
                trace_packet!("{{stdout}} {}", s);
            }
        }
    };
    let stderr = stream! {
        while let Some(Ok(s)) = stderr.next().await {
            if let Some(s) = s.strip_prefix("[TRACE] ") {
                trace_packet!("{}", s);
            } else {
                trace_packet!("{{stderr}} {}", s);
            }
        }
        // TODO: we might have to spawn this
        if false {
            yield EcamDriverOutput::Ready;
        }
    };

    let termination = stream! {
        let _ = child.wait().await;
        *alive.lock().await = false;
        yield EcamDriverOutput::Done
    };

    Result::Ok(stdout.merge(stderr).merge(termination))
}

pub async fn connect(id: &EcamId) -> Result<EcamSubprocess, EcamError> {
    let mut cmd = tokio::process::Command::new(std::env::current_exe()?);
    cmd.arg("--trace");
    cmd.arg("x-internal-pipe");
    cmd.arg("--device-name");
    cmd.arg(id.to_string());
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    let mut child = cmd.spawn()?;
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin was missing")));

    let alive = Arc::new(Mutex::new(true));
    let s = Box::pin(stream(child, alive.clone()).await?);
    Result::Ok(EcamSubprocess {
        stdin,
        receiver: EcamPacketReceiver::from_stream(s, false),
        alive,
    })
}
