use std::time::Duration;

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

pub type INodeNumber = u64;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub filter: Filter,
    pub action: Action,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TimeSlice {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Filter {
    Basic,
    Scheduled(Vec<TimeSlice>),
    TimeLimited(Duration),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Action {
    BlockProgramExecution(INodeNumber),
}
