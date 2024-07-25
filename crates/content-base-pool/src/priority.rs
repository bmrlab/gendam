use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use strum_macros::AsRefStr;

#[derive(AsRefStr, Clone, Copy, strum_macros::Display, Debug)]
pub enum TaskPriority {
    #[strum(serialize = "0")]
    Low,
    #[strum(serialize = "5")]
    Normal,
    #[strum(serialize = "10")]
    High,
}

#[derive(Copy, Clone, Debug)]
pub struct OrderedTaskPriority {
    raw: TaskPriority,
    timestamp: u128,
    insert_order: Option<usize>,
}

impl Into<OrderedTaskPriority> for TaskPriority {
    fn into(self) -> OrderedTaskPriority {
        OrderedTaskPriority {
            raw: self,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            insert_order: None,
        }
    }
}

impl From<TaskPriority> for usize {
    fn from(priority: TaskPriority) -> Self {
        priority.to_string().parse().unwrap()
    }
}

impl PartialEq for TaskPriority {
    fn eq(&self, other: &Self) -> bool {
        usize::from(*self) == usize::from(*other)
    }
}

impl Eq for TaskPriority {}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        usize::from(*self).cmp(&usize::from(*other))
    }
}

impl OrderedTaskPriority {
    pub fn with_insert_order(mut self, insert_order: usize) -> Self {
        self.insert_order = Some(insert_order);
        self
    }
}

impl Display for OrderedTaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{:?}", self.raw, self.timestamp, self.insert_order)
    }
}

impl PartialEq for OrderedTaskPriority {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw && self.timestamp == other.timestamp
    }
}

impl Eq for OrderedTaskPriority {}

impl PartialOrd for OrderedTaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedTaskPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.raw.cmp(&other.raw) {
            std::cmp::Ordering::Equal => match self.timestamp.cmp(&other.timestamp) {
                // task has higher priority when timestamp is smaller
                std::cmp::Ordering::Equal => match self.insert_order.cmp(&other.insert_order) {
                    // task has higher priority if it has been inserted earlier
                    std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
                    std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                    std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
                },
                std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            },
            other => other,
        }
    }
}
