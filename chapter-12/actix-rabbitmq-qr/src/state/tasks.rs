use std::fmt;
use indexmap::IndexMap;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use crate::queue_actor::TaskId;
use crate::QrResponse;

pub type SharedTasks = Arc<Mutex<IndexMap<String, Record>>>;

pub fn init_tasks() -> SharedTasks {
    Arc::new(Mutex::new(IndexMap::new()))
}

#[derive(Clone)]
pub struct Record {
    task_id: TaskId,
    timestamp: DateTime<Utc>,
    status: Status,
}

impl Record {
    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
}

#[derive(Clone)]
pub enum Status {
    InProgress,
    Done(QrResponse),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::InProgress => write!(f, "in progress"),
            Status::Done(resp) => match resp {
                QrResponse::Succeed(data) => write!(f, "done: {data}"),
                QrResponse::Failed(err) => write!(f, "failed: {err}"),
            },
        }
    }
}
