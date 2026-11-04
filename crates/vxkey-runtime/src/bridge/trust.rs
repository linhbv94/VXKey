//! macOS Security.framework checks, off the input loop. Development trust is an
//! explicit, immutable startup pin; no unsigned / same-UID fallback.
use serde::Deserialize;
use std::{
    ffi::{CStr, CString, c_char, c_int},
    fs,
    os::unix::{fs::MetadataExt, io::AsRawFd},
    path::Path,
};
unsafe extern "C" {
    fn vx_peer_require(fd: c_int, text: *const c_char) -> c_int;
    fn vx_own_team(output: *mut c_char, capacity: i64) -> c_int;
    fn vx_self_require(text: *const c_char) -> c_int;
}
#[derive(Clone)]
pub struct Trust {
    peer: CString,
}
impl Trust {
    pub fn load(root: &Path, companion: bool) -> Result<Self, String> {
        let own = if companion {
            "com.vxkey.desktop"
        } else {
            "com.vxkey.engine"
        };
        let peer = if companion {
            "com.vxkey.engine"
        } else {
            "com.vxkey.desktop"
        };
        #[cfg(any(debug_assertions, feature = "development-bridge"))]
        if std::env::var("VXKEY_BRIDGE_DEV_TRUST").as_deref() == Ok("1") {
            // Separate test identity exists only inside the pinned development path.
            let (own, peer) = if std::env::var("VXKEY_BRIDGE_TEST_IDENTITY").as_deref() == Ok("1") {
                if companion { ("com.vxkey.test.desktop", "com.vxkey.test.engine") }
                else { ("com.vxkey.test.engine", "com.vxkey.test.desktop") }
            } else { (own, peer) };
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Pair {
                companion: String,
                engine: String,
            }
            let path = root.join("bridge_pair.json");
            let m = fs::symlink_metadata(&path).map_err(|_| "trust_unavailable")?;
            // SAFETY: geteuid has no arguments and no ownership contract.
            if !m.is_file()
                || m.uid() != unsafe { libc::geteuid() }
                || m.mode() & 0o077 != 0
                || m.len() > 512
            {
                return Err("invalid_trust_file".into());
            }
            let file = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW)
                .open(path)
                .map_err(|_| "trust_unavailable")?;
            let pair: Pair = serde_json::from_reader(file).map_err(|_| "invalid_trust_file")?;
            for hash in [&pair.companion, &pair.engine] {
                if hash.len() != 40 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err("invalid_trust_file".into());
                }
            }
            let (own_hash, peer_hash) = if companion {
                (&pair.companion, &pair.engine)
            } else {
                (&pair.engine, &pair.companion)
            };
            let own_req = CString::new(format!("identifier \"{own}\" and cdhash H\"{own_hash}\""))
                .map_err(|_| "invalid_identity")?;
            // SAFETY: live NUL-terminated requirement; C retains no pointer.
            if unsafe { vx_self_require(own_req.as_ptr()) } != 1 {
                return Err("signature_invalid".into());
            }
            return Ok(Self {
                peer: CString::new(format!("identifier \"{peer}\" and cdhash H\"{peer_hash}\""))
                    .unwrap(),
            });
        }
        let _ = root;
        // Immutable release identity. Same certificate binds both peers across updates.
        if let Some(leaf) = option_env!("VXKEY_SIGNER_SHA1") {
            if leaf.len() != 40 || !leaf.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("invalid_identity".into()); }
            let own_req = CString::new(format!("identifier \"{own}\" and certificate leaf = H\"{leaf}\"")).unwrap();
            // SAFETY: valid NUL-terminated requirement, no retained pointer.
            if unsafe { vx_self_require(own_req.as_ptr()) } != 1 { return Err("signature_invalid".into()); }
            return Ok(Self { peer: CString::new(format!("identifier \"{peer}\" and certificate leaf = H\"{leaf}\"")).unwrap() });
        }
        let mut team = [0 as c_char; 32];
        // SAFETY: writable buffer of the passed capacity; C writes a terminated CFString.
        if unsafe { vx_own_team(team.as_mut_ptr(), team.len() as i64) } != 1 {
            return Err("trust_unavailable".into());
        }
        // SAFETY: success contract guarantees NUL within buffer.
        let team = unsafe { CStr::from_ptr(team.as_ptr()) }
            .to_str()
            .map_err(|_| "invalid_identity")?;
        if team.len() != 10
            || !team
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        {
            return Err("invalid_identity".into());
        }
        let own_req=CString::new(format!("anchor apple generic and identifier \"{own}\" and certificate leaf[subject.OU] = \"{team}\"")).unwrap();
        // SAFETY: live NUL-terminated requirement and no retained memory.
        if unsafe { vx_self_require(own_req.as_ptr()) } != 1 {
            return Err("signature_invalid".into());
        }
        Ok(Self{peer:CString::new(format!("anchor apple generic and identifier \"{peer}\" and certificate leaf[subject.OU] = \"{team}\"")).unwrap()})
    }
    pub fn check(&self, stream: &std::os::unix::net::UnixStream) -> bool {
        // SAFETY: owned, connected descriptor stays alive through a synchronous call.
        unsafe { vx_peer_require(stream.as_raw_fd(), self.peer.as_ptr()) == 1 }
    }
}
use std::os::unix::fs::OpenOptionsExt;
