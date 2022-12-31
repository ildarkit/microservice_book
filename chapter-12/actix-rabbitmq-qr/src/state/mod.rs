pub mod tasks;

use actix::Addr;
use tasks::SharedTasks;
use crate::queue_actor::{QueueActor, QueueHandler};

#[derive(Clone)]
pub struct State<T: QueueHandler> {
    pub tasks: SharedTasks,
    pub addr: Addr<QueueActor<T>>,
}

impl<T: QueueHandler> State<T> {
    pub fn new(tasks: SharedTasks, addr: Addr<QueueActor<T>>) -> Self {
        Self {
            tasks,
            addr,
        }
    } 
}
