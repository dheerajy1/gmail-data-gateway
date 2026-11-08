use jobs_api_gateway::config::env::env;

// Stage a consumed topic belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    // JOBS_WRITE_TOPIC, the Azure write
    Write,

    // JOBS_READ_TOPIC, the PostgreSQL read model
    Read,

    // JOBS_STATUS_TOPIC, the status events
    Status,
}

// The topic names of the pipeline (SPEC section 10)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topics {
    // Write-model topic
    pub write: String,

    // Read-model topic
    pub read: String,

    // Status topic
    pub status: String,

    // Dead letter topic of the write-model stage
    pub write_dlq: String,

    // Dead letter topic of the read-model stage
    pub read_dlq: String,
}

impl Topics {
    // Reads the five topic names from the env
    pub fn from_env() -> Self {
        let env = env();

        Self {
            write: env.jobs_write_topic.clone(),
            read: env.jobs_read_topic.clone(),
            status: env.jobs_status_topic.clone(),
            write_dlq: env.jobs_write_dlq_topic.clone(),
            read_dlq: env.jobs_read_dlq_topic.clone(),
        }
    }

    // The three topics the subscriber consumes, dead letter topics are only published to
    pub fn subscribed(&self) -> Vec<String> {
        vec![self.write.clone(), self.read.clone(), self.status.clone()]
    }

    // The stage of a consumed topic, None for any other topic
    pub fn route(&self, topic: &str) -> Option<Route> {
        if topic == self.write {
            Some(Route::Write)
        } else if topic == self.read {
            Some(Route::Read)
        } else if topic == self.status {
            Some(Route::Status)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Topic names used by the tests
    fn topics() -> Topics {
        Topics {
            write: "w".to_string(),
            read: "r".to_string(),
            status: "s".to_string(),
            write_dlq: "w-dlq".to_string(),
            read_dlq: "r-dlq".to_string(),
        }
    }

    // The subscriber consumes write, read and status in that order
    #[test]
    fn subscribes_to_three_topics() {
        assert_eq!(topics().subscribed(), ["w", "r", "s"]);
    }

    // Each consumed topic maps to its stage
    #[test]
    fn routes_consumed_topics() {
        let topics = topics();

        assert_eq!(topics.route("w"), Some(Route::Write));
        assert_eq!(topics.route("r"), Some(Route::Read));
        assert_eq!(topics.route("s"), Some(Route::Status));
    }

    // Dead letter and unknown topics have no route
    #[test]
    fn dead_letter_and_unknown_topics_have_no_route() {
        let topics = topics();

        assert_eq!(topics.route("w-dlq"), None);
        assert_eq!(topics.route("r-dlq"), None);
        assert_eq!(topics.route("other"), None);
    }
}
