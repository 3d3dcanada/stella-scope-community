//! The farm licence (farm board WO-09). Stella Scope is free for up to five printers. release-2026-11-14 (the owner: "Nobody
//! should be able to add more than five printers"): the limit is hard. Without a licence that covers it, a sixth real
//! printer is refused on every path (src/farm.rs `add_one` and `Farm::start`); demo printers never count. Nothing phones
//! home and there is no account: a licence is a short piece of text that a person pastes in and that is checked here,
//! offline, against a public key compiled into the program. There is no trial (a trial runs on the computer's clock,
//! which its owner can turn back).
//!
//! Format, one line: `SFL1.<payload>.<signature>`, both parts base64url without padding. The payload is
//! JSON, `{"name":"...","printers":N,"issued":"YYYY-MM-DD"}`, and the signature is Ed25519 over the
//! payload's exact bytes. Changing one character of the name or the count breaks the signature.
//! release-2026-11-14 (the owner, on Fable's advice): the payload may also carry `id`, `email`, `type` ("farm",
//! the default, or "trial") and `expiry`. A trial is a signed key emailed by 3D3D that ends on its date;
//! "unlimited" is a count of UNLIMITED. The end date is checked against the latest time the service has
//! ever seen (src/farm.rs `licence_now`), so turning the clock back does not extend it.
//!
//! The private key is never in this repository: `tools/farm-licence.rs` makes and uses it from a path
//! outside any git work tree, named by an environment variable.
use serde_json::{json, Value};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, String>;

/// Printers that need no licence and are never nagged about.
pub const FREE_PRINTERS: usize = 5;

/// Where to ask for a farm licence, in every refusal.
pub const ASK: &str = "info@3d3d.ca";

/// The printer count of an unlimited licence.
pub const UNLIMITED: u32 = 100_000;

/// The public key licences are checked against (Ed25519, 32 bytes), made by `farm-licence keygen` on
/// 2026-10-12. Its private half is in `~/.config/stella-scope/farm-licence.key` on the build machine,
/// outside the repository. Replacing it means making a new key pair and rebuilding.
#[cfg(not(feature = "test-licence-key"))]
pub const PUBLIC_KEY: [u8; 32] = [
    159, 66, 103, 158, 123, 66, 160, 18, 25, 136, 166, 181, 88, 163, 53, 133, 206, 18, 209, 17, 21,
    20, 47, 226, 113, 46, 226, 7, 5, 226, 236, 92,
];

/// The test build's key (see Cargo.toml, feature `test-licence-key`): 64 hex digits from STELLA_TEST_LICENCE_PUBKEY, read at build time.
#[cfg(feature = "test-licence-key")]
pub const PUBLIC_KEY: [u8; 32] = test_key(env!("STELLA_TEST_LICENCE_PUBKEY"));
#[cfg(feature = "test-licence-key")]
const fn test_key(hex: &str) -> [u8; 32] {
    const fn nibble(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("STELLA_TEST_LICENCE_PUBKEY must be hex digits"),
        }
    }
    let b = hex.as_bytes();
    assert!(
        b.len() == 64,
        "STELLA_TEST_LICENCE_PUBKEY must be 64 hex digits"
    );
    let mut out = [0_u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (nibble(b[2 * i]) << 4) | nibble(b[2 * i + 1]);
        i += 1;
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
pub struct Licence {
    pub name: String,
    pub printers: u32,
    pub issued: String,
    /// `YYYY-MM-DD` when the licence ends; none for a perpetual one (the normal case).
    pub expiry: Option<String>,
    /// "farm" or "trial".
    pub kind: String,
    pub id: String,
    pub email: String,
    /// The computer code the licence was issued for (the owner 2026-11-14: one licence, one computer); empty in an older licence.
    pub machine: String,
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn b64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        for i in 0..=c.len() {
            out.push(B64[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    out
}

pub fn b64_decode(s: &str) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0_u32, 0);
    for ch in s.bytes() {
        let v = B64
            .iter()
            .position(|&b| b == ch)
            .ok_or("the licence has a character that does not belong in it")?
            as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    // red team #11: leftover bits must be zero, so one licence has exactly one spelling
    if acc != 0 {
        return Err("the licence has a character that does not belong in it".into());
    }
    Ok(out)
}

/// Check a licence's text against `key`. The reasons are in words a person can act on.
pub fn verify_with(text: &str, key: &[u8; 32]) -> Result<Licence> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let mut parts = text.split('.');
    let (Some("SFL1"), Some(payload), Some(sig), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(
            "This is not a Stella Scope farm licence. It starts SFL1. and has two parts after it."
                .into(),
        );
    };
    let payload_bytes = b64_decode(payload)?;
    let sig_bytes: [u8; 64] = b64_decode(sig)?.try_into().map_err(|_| {
        "The licence is cut short or damaged: its signature is the wrong length.".to_owned()
    })?;
    let vk = VerifyingKey::from_bytes(key).map_err(|_| "This copy of Stella Scope has no licence key built in yet, so it cannot check a licence.".to_owned())?;
    vk.verify(&payload_bytes, &Signature::from_bytes(&sig_bytes))
        .map_err(|_| {
            "This licence does not check out: it was changed, or it was not made for Stella Scope."
                .to_owned()
        })?;
    let v: Value = serde_json::from_slice(&payload_bytes)
        .map_err(|_| "The licence is signed but unreadable.".to_owned())?;
    let name = v["name"]
        .as_str()
        .filter(|n| !n.trim().is_empty() && n.len() <= 120)
        .ok_or("The licence has no name in it.")?
        .to_owned();
    let printers = v["printers"]
        .as_u64()
        .filter(|&p| (1..=100_000).contains(&p))
        .ok_or("The licence does not say how many printers it covers.")? as u32;
    let issued = v["issued"].as_str().unwrap_or("").to_owned();
    let expiry = v["expiry"].as_str().map(str::to_owned);
    let kind = if v["type"] == "trial" {
        "trial"
    } else {
        "farm"
    }
    .to_owned();
    if kind == "trial" && expiry.is_none() {
        return Err("This trial licence has no end date, so it is not accepted.".into());
    }
    let text = |k: &str| {
        v[k].as_str()
            .unwrap_or("")
            .chars()
            .take(120)
            .collect::<String>()
    };
    Ok(Licence {
        name,
        printers,
        issued,
        expiry,
        kind,
        id: text("id"),
        email: text("email"),
        machine: text("machine"),
    })
}

/// The program checks with `verify_with`; this one is for `tools/farm-licence.rs` and the tests.
#[allow(dead_code)]
pub fn verify(text: &str) -> Result<Licence> {
    verify_with(text, &PUBLIC_KEY)
}

/// Make a licence. Used by `tools/farm-licence.rs` and the tests; the program itself only verifies.
/// `expiry` is optional: a perpetual licence has none (the normal case, see docs/FARM-LICENCE-TERMS.md).
#[allow(dead_code)]
pub fn issue(
    seed: &[u8; 32],
    name: &str,
    printers: u32,
    issued: &str,
    expiry: Option<&str>,
) -> String {
    let mut payload = json!({"name":name,"printers":printers,"issued":issued});
    if let Some(e) = expiry {
        payload
            .as_object_mut()
            .unwrap()
            .insert("expiry".to_owned(), json!(e));
    }
    issue_with(seed, &payload)
}

/// Sign any payload (the tool's trial and unlimited keys carry `id`, `email` and `type` too).
#[allow(dead_code)]
pub fn issue_with(seed: &[u8; 32], payload: &Value) -> String {
    use ed25519_dalek::{Signer, SigningKey};
    let payload = payload.to_string();
    let sig = SigningKey::from_bytes(seed).sign(payload.as_bytes());
    format!(
        "SFL1.{}.{}",
        b64_encode(payload.as_bytes()),
        b64_encode(&sig.to_bytes())
    )
}

#[allow(dead_code)]
pub fn public_key_of(seed: &[u8; 32]) -> [u8; 32] {
    ed25519_dalek::SigningKey::from_bytes(seed)
        .verifying_key()
        .to_bytes()
}

/// Today as `YYYY-MM-DD` (UTC) from epoch seconds, for a licence's expiry.
pub fn ymd(now: f64) -> String {
    let days = (now / 86400.0).floor() as i64;
    // civil-from-days (Howard Hinnant), proleptic Gregorian
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// A saved licence that checks out and has not ended, or why not.
pub fn valid(saved: Option<&str>, key: &[u8; 32], now: f64) -> (Option<Licence>, Option<String>) {
    valid_on(saved, key, now, None)
}

/// `valid`, and on this computer: with `machine` given, a licence issued for another computer code, or for none, is refused.
pub fn valid_on(
    saved: Option<&str>,
    key: &[u8; 32],
    now: f64,
    machine: Option<&str>,
) -> (Option<Licence>, Option<String>) {
    if let (Some(m), Some(Ok(l))) = (machine, saved.map(|t| verify_with(t, key))) {
        if l.machine.is_empty() {
            return (None, Some(format!("This licence is not tied to a computer. Write to {ASK} with this computer's code, {m}, for one that is.")));
        }
        if !l.machine.eq_ignore_ascii_case(m) {
            return (None, Some(format!("This licence is for another computer (code {}). This computer's code is {m}. To move it, write to {ASK}.", l.machine)));
        }
    }
    match saved.map(|t| verify_with(t, key)) {
        Some(Ok(l)) if l.expiry.as_deref().is_some_and(|e| e < ymd(now).as_str()) => {
            let e = l.expiry.clone().unwrap_or_default();
            (
                None,
                Some(format!("The farm licence for {} ended on {e}.", l.name)),
            )
        }
        Some(Ok(l)) => (Some(l), None),
        Some(Err(e)) => (None, Some(e)),
        None => (None, None),
    }
}

/// How many real printers this service may run. Open source (AGPL-3.0) from 0.6.3: no limit.
// ponytail: the licence-key code below is kept dormant (tests and old keys still parse); delete it when nobody holds a key.
pub fn allowed(_licence: Option<&Licence>) -> usize {
    UNLIMITED as usize
}

/// The words a person reads when a printer is refused: the limit, how to get more, and that nothing was lost.
pub fn refusal(allowed: usize) -> String {
    if allowed == FREE_PRINTERS {
        format!("Stella Scope is free for up to {FREE_PRINTERS} printers, and this farm has {FREE_PRINTERS}. To run more, get a farm licence: write to {ASK}. Nothing was changed; your printers and recordings are as they were.")
    } else {
        format!("Your farm licence covers {allowed} printers, and this farm has {allowed}. For a licence that covers more, write to {ASK}. Nothing was changed; your printers and recordings are as they were.")
    }
}

/// Where a farm of `count` real printers stands, for the pages. `count` may be over what is allowed only when the settings
/// file names more printers than that (written by hand, or restored from another farm): those beyond it are not started.
pub fn status_with_clock(saved: Option<&str>, count: usize, key: &[u8; 32], now: f64) -> Value {
    status_on(saved, count, key, now, None)
}

/// The status on this computer: the licence must be for `machine`, and the code is in the answer for the page to show.
pub fn status_on(
    saved: Option<&str>,
    count: usize,
    key: &[u8; 32],
    now: f64,
    machine: Option<&str>,
) -> Value {
    let (licence, problem) = valid_on(saved, key, now, machine);
    let allowed = allowed(licence.as_ref());
    let over = count > allowed;
    let (state, note) = if count <= FREE_PRINTERS && licence.is_none() {
        (
            "free",
            format!("Open source, no printer limit. This farm has {count}."),
        )
    } else if !over {
        let l = licence.as_ref();
        let up_to = if allowed >= UNLIMITED as usize {
            "any number of".to_owned()
        } else {
            format!("up to {allowed}")
        };
        match l.filter(|l| l.kind == "trial") {
            Some(t) => (
                "licensed",
                format!(
                    "Trial licence for {}: {up_to} printers until {}. This farm has {count}.",
                    t.name,
                    t.expiry.as_deref().unwrap_or("")
                ),
            ),
            None => (
                "licensed",
                format!(
                    "Licensed to {} for {up_to} printers. This farm has {count}.",
                    l.map_or("", |l| l.name.as_str())
                ),
            ),
        }
    } else if licence.is_some() {
        ("too_few", format!("The licence covers {allowed} printers; the settings name {count}. The first {allowed} run; the others wait until a licence covers them. Write to {ASK}."))
    } else {
        ("over", format!("Stella Scope is free for up to {FREE_PRINTERS} printers; the settings name {count}. The first {FREE_PRINTERS} run; the others wait for a farm licence. Write to {ASK}."))
    };
    json!({
        "printers":count,"free_printers":FREE_PRINTERS,"allowed":allowed,"state":state,
        "wanted":over,"locked":over,"trial":false,"full":count >= allowed,
        "licence":licence.as_ref().map(|l| json!({"name":l.name,"printers":l.printers,"issued":l.issued,"expiry":l.expiry,"type":l.kind,"id":l.id,"machine":l.machine})),
        "problem":problem,
        "note":note,
        "ask":ASK,
        "machine":machine,
    })
}

/// Where a farm of `count` printers stands, using the system clock. Tests only; the program passes its own clock.
#[cfg(test)]
pub fn status(saved: Option<&str>, count: usize, key: &[u8; 32]) -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    status_with_clock(saved, count, key, now)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u8; 32] = [7; 32];
    const OTHER: [u8; 32] = [9; 32];

    #[test]
    fn base64url_round_trips_every_length() {
        for n in 0..40 {
            let bytes: Vec<u8> = (0..n).map(|i| (i * 37 + 11) as u8).collect();
            assert_eq!(
                b64_decode(&b64_encode(&bytes)).unwrap(),
                bytes,
                "length {n}"
            );
        }
        assert!(b64_decode("not valid!").is_err());
    }

    #[test]
    fn a_valid_licence_says_who_and_how_many() {
        let key = public_key_of(&SEED);
        let text = issue(&SEED, "Maple Street Print Farm", 12, "2026-10-12", None);
        let l = verify_with(&text, &key).unwrap();
        assert_eq!(
            (l.name.as_str(), l.printers, l.issued.as_str()),
            ("Maple Street Print Farm", 12, "2026-10-12")
        );
        // Pasted with a line break or a trailing space still checks.
        assert!(verify_with(&format!("  {}\n", text.replace('.', ".\n")), &key).is_ok());
    }

    #[test]
    fn a_changed_licence_does_not_check_out() {
        let key = public_key_of(&SEED);
        let text = issue(&SEED, "Maple Street Print Farm", 12, "2026-10-12", None);
        // More printers, the same signature.
        let forged = issue(&SEED, "Maple Street Print Farm", 120, "2026-10-12", None);
        let mixed = format!(
            "SFL1.{}.{}",
            forged.split('.').nth(1).unwrap(),
            text.split('.').nth(2).unwrap()
        );
        assert!(verify_with(&mixed, &key)
            .unwrap_err()
            .contains("does not check out"));
        // One character of the name.
        let mut t = text.clone().into_bytes();
        let i = t.len() / 3;
        t[i] = if t[i] == b'A' { b'B' } else { b'A' };
        assert!(verify_with(&String::from_utf8(t).unwrap(), &key).is_err());
        // Signed by someone else's key.
        let foreign = issue(&OTHER, "Maple Street Print Farm", 12, "2026-10-12", None);
        assert!(verify_with(&foreign, &key)
            .unwrap_err()
            .contains("does not check out"));
        // Not a licence at all, and cut short.
        assert!(verify_with("hello", &key)
            .unwrap_err()
            .contains("not a Stella Scope farm licence"));
        assert!(verify_with(&text[..text.len() - 6], &key).is_err());
        assert!(verify_with("", &key).is_err());
    }

    #[test]
    fn the_compiled_in_key_is_a_real_key_and_a_test_key_does_not_pass_it() {
        assert!(
            PUBLIC_KEY != [0; 32],
            "run farm-licence keygen and paste the public key into src/licence.rs"
        );
        assert!(ed25519_dalek::VerifyingKey::from_bytes(&PUBLIC_KEY).is_ok());
        // A licence made with a throwaway key is not accepted by the built-in one.
        let text = issue(&SEED, "x", 6, "2026-10-12", None);
        assert!(verify(&text).unwrap_err().contains("does not check out"));
    }
}
