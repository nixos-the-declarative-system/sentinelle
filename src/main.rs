use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use openssl::asn1::Asn1Time;
use openssl::nid::Nid;
use openssl::x509::X509;

/// `sentinelle 2.8.0 (linux/amd64)`: the version style of vendor CLIs,
/// os/arch included.
fn short_version() -> &'static str {
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };
    Box::leak(
        format!(
            "{} ({}/{})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            arch
        )
        .into_boxed_str(),
    )
}

#[derive(Parser)]
#[command(
    name = "sentinelle",
    version = short_version(),
    about = "CLI for the Sentinelle monitoring platform",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Probe one or more HTTPS targets: availability, latency, certificate
    Check {
        /// Host to probe, optional port (e.g. example.org or example.org:8443)
        #[arg(required = true)]
        targets: Vec<String>,

        /// JSON output, one line per target
        #[arg(long)]
        json: bool,

        /// Maximum time per probe, in seconds
        #[arg(long, default_value_t = 10)]
        timeout: u64,

        /// Warning threshold for certificate expiry, in days
        #[arg(long, value_name = "DAYS", default_value_t = 14)]
        cert_warning: i32,
    },
}

/// Result of a successful probe (the connection went through; the
/// service may still be in a warning state).
struct Probe {
    http_status: u16,
    latency_ms: u128,
    cert_days: Option<i32>,
    cert_expires: Option<String>,
    cert_issuer: Option<String>,
}

fn run_probe(target: &str, timeout: Duration) -> Result<Probe, String> {
    let url = format!("https://{target}/");
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .tls_info(true)
        .user_agent(concat!("sentinelle/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;

    let start = Instant::now();
    let response = client.get(&url).send().map_err(root_cause)?;
    let latency_ms = start.elapsed().as_millis();

    let mut probe = Probe {
        http_status: response.status().as_u16(),
        latency_ms,
        cert_days: None,
        cert_expires: None,
        cert_issuer: None,
    };

    // The certificate presented by the peer, as reqwest saw it (DER),
    // read back by OpenSSL to extract its expiry date and its issuer.
    if let Some(der) = response
        .extensions()
        .get::<reqwest::tls::TlsInfo>()
        .and_then(|t| t.peer_certificate())
    {
        if let Ok(cert) = X509::from_der(der) {
            let now = Asn1Time::days_from_now(0).expect("clock");
            if let Ok(delta) = now.diff(cert.not_after()) {
                probe.cert_days = Some(delta.days);
            }
            probe.cert_expires = Some(cert.not_after().to_string());
            probe.cert_issuer = cert
                .issuer_name()
                .entries_by_nid(Nid::COMMONNAME)
                .next()
                .and_then(|e| e.data().to_string().ok());
        }
    }

    Ok(probe)
}

/// reqwest errors chain all their causes; keep only the last one, the
/// only one that tells an operator anything.
fn root_cause(e: reqwest::Error) -> String {
    let mut cause: &dyn std::error::Error = &e;
    while let Some(source) = cause.source() {
        cause = source;
    }
    cause.to_string()
}

fn print_text(target: &str, result: &Result<Probe, String>, threshold: i32) {
    match result {
        Ok(p) => {
            let state = if p.http_status < 400 {
                "online"
            } else {
                "warning"
            };
            println!("● {target} — {state}");
            println!("  HTTP {} in {} ms", p.http_status, p.latency_ms);
            match (p.cert_days, &p.cert_expires) {
                (Some(days), Some(expires)) => {
                    let issuer = p
                        .cert_issuer
                        .as_deref()
                        .map(|cn| format!(", issued by {cn}"))
                        .unwrap_or_default();
                    let unit = if days == 1 { "day" } else { "days" };
                    if days < threshold {
                        println!("  ⚠ certificate: expires in {days} {unit} ({expires}{issuer})");
                    } else {
                        println!("  certificate: {days} {unit} left ({expires}{issuer})");
                    }
                }
                _ => println!("  certificate: unavailable"),
            }
        }
        Err(reason) => {
            println!("● {target} — offline");
            println!("  {reason}");
        }
    }
}

fn print_json(target: &str, result: &Result<Probe, String>, threshold: i32) {
    let line = match result {
        Ok(p) => serde_json::json!({
            "target": target,
            "online": true,
            "http": p.http_status,
            "latency_ms": p.latency_ms,
            "certificate_days": p.cert_days,
            "certificate_expires": p.cert_expires,
            "certificate_issuer": p.cert_issuer,
            "warning": p.http_status >= 400
                || p.cert_days.map(|d| d < threshold).unwrap_or(false),
        }),
        Err(reason) => serde_json::json!({
            "target": target,
            "online": false,
            "reason": reason,
        }),
    };
    println!("{line}");
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Command::Check {
        targets,
        json,
        timeout,
        cert_warning,
    } = cli.command;
    let timeout = Duration::from_secs(timeout);

    // Exit code = worst state encountered: 0 online, 1 warning,
    // 2 offline (the convention of monitoring probes).
    let mut worst = 0u8;
    for target in &targets {
        let result = run_probe(target, timeout);
        if json {
            print_json(target, &result, cert_warning);
        } else {
            print_text(target, &result, cert_warning);
        }
        let state = match &result {
            Ok(p) if p.http_status >= 400 => 1,
            Ok(p) if p.cert_days.map(|d| d < cert_warning).unwrap_or(false) => 1,
            Ok(_) => 0,
            Err(_) => 2,
        };
        worst = worst.max(state);
    }
    ExitCode::from(worst)
}
