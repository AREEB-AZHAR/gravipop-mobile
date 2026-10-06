use super::leaderboard::LeaderboardEntry;
use serde::Deserialize;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

const DEFAULT_URL: &str = "https://graviity-zeta.vercel.app/api/leaderboard";
pub enum Request {
    Refresh,
    Submit {
        display_name: String,
        high_score: u64,
    },
}
type Reply = Result<(Vec<LeaderboardEntry>, String), String>;

pub struct Worker {
    requests: Sender<Request>,
    pub results: Receiver<Reply>,
}

impl Default for Worker {
    fn default() -> Self {
        let endpoint = std::env::var("GRAVIPOP_LEADERBOARD_URL")
            .ok()
            .or_else(|| option_env!("GRAVIPOP_LEADERBOARD_URL").map(str::to_owned))
            .unwrap_or_else(|| DEFAULT_URL.to_string());
        Self::new(endpoint)
    }
}

impl Worker {
    fn new(endpoint: String) -> Self {
        let (requests, incoming) = mpsc::channel();
        let (outgoing, results) = mpsc::channel();
        let failure = outgoing.clone();
        // One worker preserves request order and never blocks the render thread.
        if std::thread::Builder::new()
            .name("gravipop-leaderboard".into())
            .spawn(move || {
                let agent = ureq::Agent::config_builder()
                    .timeout_global(Some(Duration::from_secs(12)))
                    .build()
                    .new_agent();
                for request in incoming {
                    let reply = execute(&agent, &endpoint, request);
                    if outgoing.send(reply).is_err() {
                        break;
                    }
                }
            })
            .is_err()
        {
            let _ = failure.send(Err(
                "Leaderboard worker unavailable. Your score is saved on this device.".into(),
            ));
        }
        Self { requests, results }
    }

    pub fn send(&self, request: Request) {
        let _ = self.requests.send(request);
    }
}

#[derive(Deserialize)]
struct Submission {
    success: bool,
    scores: Vec<LeaderboardEntry>,
}

fn execute(agent: &ureq::Agent, endpoint: &str, request: Request) -> Reply {
    let submitting = matches!(request, Request::Submit { .. });
    let result: Result<Vec<LeaderboardEntry>, String> = (|| {
        let mut response = match request {
            Request::Refresh => agent
                .get(endpoint)
                .header("Accept", "application/json")
                .call(),
            Request::Submit {
                display_name,
                high_score,
            } => agent
                .post(endpoint)
                .header("Accept", "application/json")
                .send_json(
                    serde_json::json!({ "display_name": display_name, "high_score": high_score }),
                ),
        }
        .map_err(|error| error.to_string())?;
        let body = response
            .body_mut()
            .with_config()
            .limit(256 * 1024)
            .read_to_string()
            .map_err(|error| error.to_string())?;
        if submitting {
            let submission: Submission =
                serde_json::from_str(&body).map_err(|error| error.to_string())?;
            if !submission.success {
                return Err("Score was not accepted".into());
            }
            Ok(submission.scores)
        } else {
            serde_json::from_str(&body).map_err(|error| error.to_string())
        }
    })();
    match result {
        Ok(mut entries) => {
            entries.retain(|entry| !entry.display_name.trim().is_empty());
            entries.sort_by(|a, b| b.high_score.cmp(&a.high_score));
            entries.truncate(20);
            Ok((
                entries,
                if submitting {
                    "Score submitted globally!"
                } else {
                    "Global leaderboard synchronized"
                }
                .into(),
            ))
        }
        Err(_) => Err(if submitting {
            "Score saved on this device. Cloud sync failed; check your connection."
        } else {
            "Leaderboard unavailable. Check your connection and tap Refresh."
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };

    #[test]
    fn native_reads_and_submits_the_shared_api_contract() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/api/leaderboard", server.local_addr().unwrap());
        let handler = std::thread::spawn(move || {
            for expected in ["GET", "POST"] {
                let (mut stream, _) = server.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let (header_end, content_length) = loop {
                    let mut chunk = [0; 2048];
                    let count = stream.read(&mut chunk).unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&chunk[..count]);
                    if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        assert!(headers.starts_with(&format!("{expected} /api/leaderboard ")));
                        let length = headers
                            .lines()
                            .find_map(|line| {
                                let (key, value) = line.split_once(':')?;
                                key.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        break (end + 4, length);
                    }
                };
                while bytes.len() < header_end + content_length {
                    let mut chunk = [0; 2048];
                    let count = stream.read(&mut chunk).unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&chunk[..count]);
                }
                if expected == "POST" {
                    let value: serde_json::Value =
                        serde_json::from_slice(&bytes[header_end..]).unwrap();
                    assert_eq!(value["display_name"], "Native Pilot");
                    assert_eq!(value["high_score"], 1234);
                }
                let scores = r#"[{"display_name":"Web Pilot","high_score":500},{"display_name":"Native Pilot","high_score":1234}]"#;
                let body = if expected == "POST" {
                    format!(r#"{{"success":true,"scores":{scores}}}"#)
                } else {
                    scores.into()
                };
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
            }
        });
        let worker = Worker::new(endpoint);
        worker.send(Request::Refresh);
        worker.send(Request::Submit {
            display_name: "Native Pilot".into(),
            high_score: 1234,
        });
        for _ in 0..2 {
            let (scores, _) = worker
                .results
                .recv_timeout(Duration::from_secs(5))
                .unwrap()
                .unwrap();
            assert_eq!(scores[0].display_name, "Native Pilot");
            assert_eq!(scores[1].display_name, "Web Pilot");
        }
        handler.join().unwrap();
    }

    #[test]
    fn failed_native_sync_reports_offline_without_fabricating_scores() {
        let worker = Worker::new("http://127.0.0.1:0/api/leaderboard".into());
        worker.send(Request::Refresh);
        assert!(worker
            .results
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .is_err());
    }

    #[test]
    #[ignore = "uses the deployed shared HTTPS endpoint"]
    fn deployed_global_leaderboard_is_readable_from_native_builds() {
        let worker = Worker::default();
        worker.send(Request::Refresh);
        let (entries, _) = worker
            .results
            .recv_timeout(Duration::from_secs(20))
            .unwrap()
            .unwrap();
        assert!(entries.len() <= 20);
    }
}
