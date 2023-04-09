pub enum LogType {
    AsyncMessage,
    EngineUpdate,
    Detail,
    MaxMemHit,
    Performance,
}

const ASYNC_MESSAGE: bool = false;
const ENGINE_UPDATE: bool = true;
const DETAIL: bool = false;
const MAX_MEM_HIT: bool = false;
const PERFORMANCE: bool = true;

pub fn log_message(log_type: LogType, msg: String) {
    let should_print = match log_type {
        LogType::AsyncMessage => ASYNC_MESSAGE,
        LogType::EngineUpdate => ENGINE_UPDATE,
        LogType::Detail => DETAIL,
        LogType::MaxMemHit => MAX_MEM_HIT,
        LogType::Performance => PERFORMANCE,
    };

    if should_print {
        println!("{}", msg);
    }
}
