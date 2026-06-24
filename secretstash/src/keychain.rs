use zeroize::Zeroizing;

const SERVICE: &str = "stash";
const ACCOUNT: &str = "session";

/// Save `content` to the system keychain. Returns true if saved successfully.
#[must_use]
pub fn save(content: &str) -> bool {
    platform::save(content)
}

/// Load session content from the system keychain.
/// Returns None when the entry is absent or the backend is unavailable.
#[must_use]
pub fn load() -> Option<Zeroizing<String>> {
    platform::load()
}

/// Remove the session entry from the system keychain (best effort).
pub fn clear() {
    platform::clear();
}

// ── macOS — Keychain via the Security framework ───────────────────────────
//
// Uses the SecKeychain API directly rather than shelling out to the `security`
// CLI. The CLI takes the secret as a `-w <password>` argument, which is visible
// to other processes via `ps`; the API keeps the key material out of any argv.

#[cfg(target_os = "macos")]
mod platform {
    use super::{ACCOUNT, SERVICE};
    use security_framework::passwords::{
        delete_generic_password, get_generic_password, set_generic_password,
    };
    use zeroize::Zeroizing;

    pub fn save(content: &str) -> bool {
        set_generic_password(SERVICE, ACCOUNT, content.as_bytes()).is_ok()
    }

    pub fn load() -> Option<Zeroizing<String>> {
        let bytes = get_generic_password(SERVICE, ACCOUNT).ok()?;
        if bytes.is_empty() {
            return None;
        }
        Some(Zeroizing::new(String::from_utf8(bytes).ok()?))
    }

    pub fn clear() {
        let _ = delete_generic_password(SERVICE, ACCOUNT);
    }
}

// ── Linux — Secret Service via `secret-tool` ──────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use super::{ACCOUNT, SERVICE};
    use std::io::Write;
    use std::process::{Command, Stdio};
    use zeroize::Zeroizing;

    pub fn save(content: &str) -> bool {
        let Ok(mut child) = Command::new("secret-tool")
            .args([
                "store",
                "--label=stash session key",
                "service",
                SERVICE,
                "username",
                ACCOUNT,
            ])
            .stdin(Stdio::piped())
            .spawn()
        else {
            return false;
        };
        let wrote = child
            .stdin
            .take()
            .and_then(|mut s| s.write_all(content.as_bytes()).ok())
            .is_some();
        wrote && child.wait().is_ok_and(|s| s.success())
    }

    pub fn load() -> Option<Zeroizing<String>> {
        let out = Command::new("secret-tool")
            .args(["lookup", "service", SERVICE, "username", ACCOUNT])
            .output()
            .ok()?;
        if out.status.success() && !out.stdout.is_empty() {
            Some(Zeroizing::new(String::from_utf8(out.stdout).ok()?))
        } else {
            None
        }
    }

    pub fn clear() {
        let _ = Command::new("secret-tool")
            .args(["clear", "service", SERVICE, "username", ACCOUNT])
            .status();
    }
}

// ── All other platforms — no keychain support ─────────────────────────────

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod platform {
    use zeroize::Zeroizing;

    pub fn save(_: &str) -> bool {
        false
    }
    pub fn load() -> Option<Zeroizing<String>> {
        None
    }
    pub fn clear() {}
}
