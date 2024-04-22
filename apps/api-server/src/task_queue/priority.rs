use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};
use file_handler::TaskPriority as TaskPriorityRaw;

#[derive(Clone, Copy, Debug)]
pub struct TaskPriority {
    priority: TaskPriorityRaw,
    timestamp: u128,
    insert_order: Option<usize>,
}

impl TaskPriority {
    pub fn new(priority: TaskPriorityRaw) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        Self {
            priority,
            timestamp,
            insert_order: None,
        }
    }

    pub fn with_insert_order(mut self, insert_order: usize) -> Self {
        self.insert_order = Some(insert_order);
        self
    }
}

impl Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{:?}",
            self.priority, self.timestamp, self.insert_order
        )
    }
}

impl PartialEq for TaskPriority {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
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
        match self.priority.cmp(&other.priority) {
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
