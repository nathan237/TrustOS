//! Minimal TLS client over the existing TCP stack.

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::min;

use mbedtls::ssl::{Config, Context, Io as SslIo};
use mbedtls::ssl::config::{AuthMode, Endpoint, Transport, Preset};
use mbedtls::error::{Error as MbedtlsError, HiError, LoError};
use mbedtls::alloc::List as MbedtlsList;
use mbedtls::x509::{Certificate, VerifyError};

pub struct TcpStream {
    dest_ip: [u8; 4],
    dest_port: u16,
    src_port: u16,
    timeout_ms: u32,
    rx_cache: Vec<u8>,
}

impl TcpStream {
    pub fn connect(dest_ip: [u8; 4], dest_port: u16, timeout_ms: u32) -> Result<Self, &'static str> {
        let src_port = crate::netstack::tcp::send_syn(dest_ip, dest_port)?;
        let established = crate::netstack::tcp::wait_for_established(dest_ip, dest_port, src_port, timeout_ms);
        if !established {
            return Err("TLS TCP connect timeout");
        }

        Ok(Self {
            dest_ip,
            dest_port,
            src_port,
            timeout_ms,
            rx_cache: Vec::new(),
        })
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        let _ = crate::netstack::tcp::send_fin(self.dest_ip, self.dest_port, self.src_port);
    }
}

impl SslIo for TcpStream {
    fn recv(&mut self, buf: &mut [u8]) -> mbedtls::error::Result<usize> {
        if !self.rx_cache.is_empty() {
            let n = min(buf.len(), self.rx_cache.len());
            buf[..n].copy_from_slice(&self.rx_cache[..n]);
            self.rx_cache.drain(..n);
            return Ok(n);
        }

        let start = crate::logger::get_ticks();
        loop {
            crate::netstack::poll();

            if let Some(data) = crate::netstack::tcp::recv_data(self.dest_ip, self.dest_port, self.src_port) {
                if data.is_empty() {
                    continue;
                }

                let n = min(buf.len(), data.len());
                buf[..n].copy_from_slice(&data[..n]);
                if n < data.len() {
                    self.rx_cache.extend_from_slice(&data[n..]);
                }
                return Ok(n);
            }

            if crate::netstack::tcp::fin_received(self.dest_ip, self.dest_port, self.src_port) {
                return Ok(0);
            }

            if crate::logger::get_ticks().saturating_sub(start) > self.timeout_ms as u64 {
                return Err(HiError::SslTimeout.into());
            }

            x86_64::instructions::hlt();
        }
    }

    fn send(&mut self, buf: &[u8]) -> mbedtls::error::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        crate::netstack::tcp::send_payload(self.dest_ip, self.dest_port, self.src_port, buf)
            .map_err(|_| MbedtlsError::from(LoError::NetSendFailed))?;
        Ok(buf.len())
    }
}

const ISRG_ROOT_X1_PEM: &[u8] = b"-----BEGIN CERTIFICATE-----\nMIIFazCCA1OgAwIBAgIRAIIQz7DSQONZRGPgu2OCiwAwDQYJKoZIhvcNAQELBQAw\nTzELMAkGA1UEBhMCVVMxKTAnBgNVBAoTIEludGVybmV0IFNlY3VyaXR5IFJlc2Vh\ncmNoIEdyb3VwMRUwEwYDVQQDEwxJU1JHIFJvb3QgWDEwHhcNMTUwNjA0MTEwNDM4\nWhcNMzUwNjA0MTEwNDM4WjBPMQswCQYDVQQGEwJVUzEpMCcGA1UEChMgSW50ZXJu\nZXQgU2VjdXJpdHkgUmVzZWFyY2ggR3JvdXAxFTATBgNVBAMTDElTUkcgUm9vdCBY\nMTCCAiIwDQYJKoZIhvcNAQEBBQADggIPADCCAgoCggIBAK3oJHP0FDfzm54rVygc\nh77ct984kIxuPOZXoHj3dcKi/vVqbvYATyjb3miGbESTtrFj/RQSa78f0uoxmyF+\n0TM8ukj13Xnfs7j/EvEhmkvBioZxaUpmZmyPfjxwv60pIgbz5MDmgK7iS4+3mX6U\nA5/TR5d8mUgjU+g4rk8Kb4Mu0UlXjIB0ttov0DiNewNwIRt18jA8+o+u3dpjq+sW\nT8KOEUt+zwvo/7V3LvSye0rgTBIlDHCNAymg4VMk7BPZ7hm/ELNKjD+Jo2FR3qyH\nB5T0Y3HsLuJvW5iB4YlcNHlsdu87kGJ55tukmi8mxdAQ4Q7e2RCOFvu396j3x+UC\nB5iPNgiV5+I3lg02dZ77DnKxHZu8A/lJBdiB3QW0KtZB6awBdpUKD9jf1b0SHzUv\nKBds0pjBqAlkd25HN7rOrFleaJ1/ctaJxQZBKT5ZPt0m9STJEadao0xAH0ahmbWn\nOlFuhjuefXKnEgV4We0+UXgVCwOPjdAvBbI+e0ocS3MFEvzG6uBQE3xDk3SzynTn\njh8BCNAw1FtxNrQHusEwMFxIt4I7mKZ9YIqioymCzLq9gwQbooMDQaHWBfEbwrbw\nqHyGO0aoSCqI3Haadr8faqU9GY/rOPNk3sgrDQoo//fb4hVC1CLQJ13hef4Y53CI\nrU7m2Ys6xt0nUW7/vGT1M0NPAgMBAAGjQjBAMA4GA1UdDwEB/wQEAwIBBjAPBgNV\nHRMBAf8EBTADAQH/MB0GA1UdDgQWBBR5tFnme7bl5AFzgAiIyBpY9umbbjANBgkq\nhkiG9w0BAQsFAAOCAgEAVR9YqbyyqFDQDLHYGmkgJykIrGF1XIpu+ILlaS/V9lZL\nubhzEFnTIZd+50xx+7LSYK05qAvqFyFWhfFQDlnrzuBZ6brJFe+GnY+EgPbk6ZGQ\n3BebYhtF8GaV0nxvwuo77x/Py9auJ/GpsMiu/X1+mvoiBOv/2X/qkSsisRcOj/KK\nNFtY2PwByVS5uCbMiogziUwthDyC3+6WVwW6LLv3xLfHTjuCvjHIInNzktHCgKQ5\nORAzI4JMPJ+GslWYHb4phowim57iaztXOoJwTdwJx4nLCgdNbOhdjsnvzqvHu7Ur\nTkXWStAmzOVyyghqpZXjFaH3pO3JLF+l+/+sKAIuvtd7u+Nxe5AW0wdeRlN8NwdC\njNPElpzVmbUq4JUagEiuTDkHzsxHpFKVK7q4+63SM1N95R1NbdWhscdCb+ZAJzVc\noyi3B43njTOQ5yOf+1CceWxG1bQVs5ZufpsMljq4Ui0/1lvh+wjChP4kqKOJ2qxq\n4RgqsahDYVvTH9w7jXbyLeiNdd8XM2w9U/t7y0Ff/9yi0GE44Za4rF2LN9d11TPA\nmRGunUHBcnWEvgJBQl9nJEiU0Zsnvgc/ubhPgXRR4Xq37Z0j4r7g1SgEEzwxA57d\nemyPxgcYxn/eR44/KJ4EBs+lVDR3veyJm+kXQ99b21/+jh5Xos1AnX5iItreGCc=\n-----END CERTIFICATE-----\n\0";

fn rtc_seems_valid() -> bool {
    let dt = crate::rtc::read_rtc();
    (2024..=2100).contains(&dt.year)
}

fn load_root_certs() -> Result<Arc<MbedtlsList<Certificate>>, &'static str> {
    let ca_list = Certificate::from_pem_multiple(ISRG_ROOT_X1_PEM)
        .map_err(|_| "Failed to parse root CA")?;
    Ok(Arc::new(ca_list))
}

fn tls_config() -> Result<Arc<Config>, &'static str> {
    let mut config = Config::new(Endpoint::Client, Transport::Stream, Preset::Default);
    config.set_authmode(AuthMode::Required);

    let rng = Arc::new(|data: *mut u8, len: usize| -> core::ffi::c_int {
        if data.is_null() {
            return -1;
        }
        let buf = unsafe { core::slice::from_raw_parts_mut(data, len) };
        crate::rng::fill_bytes(buf);
        0
    });
    config.set_rng(rng);

    let ca_list = load_root_certs()?;
    config.set_ca_list(ca_list, None);

    let allow_time_skew = !rtc_seems_valid();
    config.set_verify_callback(move |_certs, _depth, flags: &mut VerifyError| {
        if allow_time_skew {
            flags.remove(VerifyError::CERT_EXPIRED | VerifyError::CERT_FUTURE);
            flags.remove(VerifyError::CRL_EXPIRED | VerifyError::CRL_FUTURE);
        }
        Ok(())
    });

    Ok(Arc::new(config))
}

pub fn https_get(hostname: &str, ip: [u8; 4], port: u16, path: &str, host_header: &str) -> Result<usize, &'static str> {
    let stream = TcpStream::connect(ip, port, 8000)?;
    let config = tls_config()?;
    let mut ctx = Context::new(config);

    if let Err(e) = ctx.establish(stream, Some(hostname)) {
        crate::serial_println!("[TLS] handshake failed: {}", e);
        return Err("TLS handshake failed");
    }

    if let Err(e) = ctx.verify_result() {
        crate::serial_println!("[TLS] verify failed: {:?}", e);
        return Err("TLS verify failed");
    }

    let mut request = String::new();
    request.push_str("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: ");
    request.push_str(host_header);
    request.push_str("\r\nConnection: close\r\n\r\n");

    let mut sent = 0usize;
    let req_bytes = request.as_bytes();
    while sent < req_bytes.len() {
        let n = ctx.send(&req_bytes[sent..]).map_err(|_| "TLS send failed")?;
        if n == 0 {
            break;
        }
        sent += n;
    }

    let mut total = 0usize;
    let mut buf = [0u8; 1024];
    loop {
        match ctx.recv(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if let Ok(text) = core::str::from_utf8(&buf[..n]) {
                    crate::print!("{}", text);
                } else {
                    crate::println!("<binary data>");
                }
            }
            Err(e) => {
                crate::serial_println!("[TLS] recv error: {}", e);
                break;
            }
        }
    }

    ctx.close();
    Ok(total)
}
