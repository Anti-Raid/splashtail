use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Helper struct to run an event as a subprocess
///
/// Usage:
///
/// ```rust
/// use std::sync::LazyLock;
///
/// pub static SUBPROCESS_EVENT: LazyLock<SubprocessEvent> = LazyLock::new(
/// || SubprocessEvent::new(tokio::process::Command::new("python3").arg("subprocess_event.py"))
/// );
///
/// // In the event handler
/// SUBPROCESS_EVENT.push(event)?;
/// ```
pub struct SubprocessEvent {
    cmd: Mutex<(Command, Option<Child>)>,
}

impl SubprocessEvent {
    pub fn new(cmd: tokio::process::Command) -> Self {
        let mut cmd = cmd;
        cmd.stdin(std::process::Stdio::piped());

        Self {
            cmd: Mutex::new((cmd, None)),
        }
    }

    /// Push a new event to the subprocess
    pub async fn push(&self, event: &[u8]) -> Result<(), std::io::Error> {
        let mut cmd = self.cmd.lock().await;

        if cmd.1.is_none() || cmd.1.as_mut().unwrap().try_wait()?.is_some() {
            let spawned_child = cmd.0.spawn()?;
            cmd.1 = Some(spawned_child);
        }

        let child = cmd.1.as_mut().unwrap();
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(event).await?;

        Ok(())
    }
}

/// Usage:
///
/// ```rust
/// pub static SUBPROCESS_EVENT = LazyLock::new(|| SubprocessBytes {
///    data: include_bytes!("subprocess_event.py").to_vec(),
///    exec: vec!["python3".to_string(), "subprocess_event.py".to_string()],
///    filename: "subprocess_event.py".to_string(),
/// }
/// .create_subprocess().expect("Failed to create subprocess")
/// );
/// ```
///
pub struct SubprocessBytes {
    pub data: Vec<u8>,
    pub exec: Vec<String>,
    pub filename: String,
}

impl SubprocessBytes {
    pub fn create_subprocess(self) -> Result<SubprocessEvent, std::io::Error> {
        if self.exec.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No exec provided",
            ));
        }

        // Get temp dir
        let temp_dir = std::env::temp_dir();

        // Make random folder name here
        let folder_name = format!("ar-{}", botox::crypto::gen_random(64));

        // Make the folder
        let folder_path = temp_dir.join(&folder_name);

        std::fs::create_dir(&folder_path)?;

        // Make the file
        let file_path = folder_path.join(&self.filename);

        std::fs::write(&file_path, &self.data)?;

        // Make the command
        let mut cmd = Command::new(&self.exec[0]);

        for arg in &self.exec[1..] {
            if arg == "{}" {
                cmd.arg(&file_path);
            } else {
                cmd.arg(arg);
            }
        }

        Ok(SubprocessEvent::new(cmd))
    }
}
