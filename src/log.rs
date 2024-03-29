use std::time::Instant;

pub enum LogType {
    AsyncMessage,
    EngineUpdate,
    Detail,
    MaxMemHit,
    Performance,
    MoveScores,
}

const TESTING: bool = false;

const ASYNC_MESSAGE: bool = false;
const ENGINE_UPDATE: bool = false;
const DETAIL: bool = false;
const MAX_MEM_HIT: bool = true;
const PERFORMANCE: bool = false;
const MOVE_SCORES: bool = true;

pub fn log_message(log_type: LogType, msg: String) {
    let should_print = match log_type {
        LogType::AsyncMessage => ASYNC_MESSAGE,
        LogType::EngineUpdate => ENGINE_UPDATE,
        LogType::Detail => DETAIL,
        LogType::MaxMemHit => MAX_MEM_HIT,
        LogType::Performance => PERFORMANCE,
        LogType::MoveScores => MOVE_SCORES,
    };

    if should_print && !TESTING {
        println!("{}", msg);
    }
}

pub struct PerfTimer {
    start: Instant,
    label: String,
}

impl PerfTimer {
    pub fn start(label: &str) -> PerfTimer {
        PerfTimer {
            start: Instant::now(),
            label: label.to_owned(),
        }
    }

    pub fn stop(&self) {
        log_message(
            LogType::Performance,
            format!("{} - {}", self.label, self.start.elapsed().as_secs_f32()),
        );
    }
}
