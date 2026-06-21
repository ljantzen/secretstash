use anyhow::{Result, anyhow};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Clone, Copy)]
enum Backend {
    Pbcopy,
    WlCopy,
    Xclip,
    Xsel,
    ClipExe,
}

impl Backend {
    const ALL: &'static [Backend] = &[
        Backend::Pbcopy,
        Backend::WlCopy,
        Backend::Xclip,
        Backend::Xsel,
        Backend::ClipExe,
    ];

    fn cmd(self) -> &'static str {
        match self {
            Backend::Pbcopy => "pbcopy",
            Backend::WlCopy => "wl-copy",
            Backend::Xclip => "xclip",
            Backend::Xsel => "xsel",
            Backend::ClipExe => "clip.exe",
        }
    }

    fn copy_args(self) -> &'static [&'static str] {
        match self {
            Backend::Pbcopy => &[],
            Backend::WlCopy => &[],
            Backend::Xclip => &["-selection", "clipboard"],
            Backend::Xsel => &["--clipboard", "--input"],
            Backend::ClipExe => &[],
        }
    }

    // Shell fragment used by the background clear process.
    fn clear_fragment(self) -> &'static str {
        match self {
            Backend::Pbcopy => "printf '' | pbcopy",
            Backend::WlCopy => "wl-copy --clear",
            Backend::Xclip => "printf '' | xclip -selection clipboard",
            Backend::Xsel => "xsel --clipboard --clear",
            Backend::ClipExe => "echo.|clip",
        }
    }
}

/// A handle to the clipboard backend that was used for a successful copy.
/// Retains which backend was detected so `schedule_clear` uses the same tool.
pub struct Clipboard(Backend);

impl Clipboard {
    /// Copy `text` to the system clipboard.
    /// Tries backends in order and returns a handle on the first one that works.
    pub fn copy(text: &str) -> Result<Self> {
        for &backend in Backend::ALL {
            match Command::new(backend.cmd())
                .args(backend.copy_args())
                .stdin(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    child.stdin.take().unwrap().write_all(text.as_bytes())?;
                    child.wait()?;
                    return Ok(Clipboard(backend));
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            }
        }

        Err(anyhow!(
            "No clipboard command found. \
             Install pbcopy (macOS), wl-clipboard (Wayland), xclip/xsel (X11), \
             or use Windows where clip.exe is built in."
        ))
    }

    /// Spawn a detached background process that clears the clipboard after
    /// `after_secs` seconds. Errors are silently ignored — the clear is
    /// best-effort and must not interfere with the main command's exit.
    pub fn schedule_clear(&self, after_secs: u64) {
        let fragment = self.0.clear_fragment();

        #[cfg(windows)]
        let _ = Command::new("cmd")
            .args([
                "/C",
                &format!("timeout /t {} /nobreak >nul & {}", after_secs, fragment),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        #[cfg(not(windows))]
        let _ = Command::new("sh")
            .args(["-c", &format!("sleep {}; {}", after_secs, fragment)])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}
