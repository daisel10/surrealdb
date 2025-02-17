use crate::dbs::node::Timestamp;
use crate::sql;
use sql::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_arch = "wasm32")]
use wasmtimer::std::{SystemTime, UNIX_EPOCH};

// Traits cannot have async and we need sized structs for Clone + Send + Sync
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum SizedClock {
	System(SystemClock),
	#[cfg(test)]
	Fake(FakeClock),
	#[cfg(test)]
	Inc(IncFakeClock),
}

impl SizedClock {
	pub async fn now(&mut self) -> Timestamp {
		match self {
			SizedClock::System(c) => c.now(),
			#[cfg(test)]
			SizedClock::Fake(c) => c.now().await,
			#[cfg(test)]
			SizedClock::Inc(c) => c.now().await,
		}
	}
}

/// FakeClock is a clock that is fully controlled externally.
/// Use this clock for when you are testing timestamps.
#[derive(Clone, Copy)]
pub struct FakeClock {
	// Locks necessary for Send
	now: Timestamp,
}

#[allow(dead_code)]
impl FakeClock {
	pub fn new(now: Timestamp) -> Self {
		FakeClock {
			now,
		}
	}

	pub async fn now(&self) -> Timestamp {
		self.now
	}

	pub async fn set(&mut self, timestamp: Timestamp) {
		self.now = timestamp;
	}
}

/// IncFakeClock increments a local clock every time the clock is accessed, similar to a real clock.
/// This is useful when you need unique and partially deterministic timestamps for tests.
/// Partially deterministic, because you do not have direct control over how many times a clock
/// is accessed, and due to the nature of async - you neither have order guarantee.
#[derive(Clone, Copy)]
pub struct IncFakeClock {
	// Locks necessary for Send
	now: Timestamp,
	increment: Duration,
}

#[allow(dead_code)]
impl IncFakeClock {
	pub fn new(now: Timestamp, increment: Duration) -> Self {
		IncFakeClock {
			now,
			increment,
		}
	}

	pub async fn now(&mut self) -> Timestamp {
		self.now = &self.now + &self.increment;
		self.now
	}
}

/// SystemClock is a clock that uses the system time.
/// Use this when there are no other alternatives.
#[derive(Clone, Copy)]
pub struct SystemClock;

impl SystemClock {
	pub fn new() -> Self {
		SystemClock
	}
	pub fn now(&self) -> Timestamp {
		// Use a timestamp oracle if available
		let now: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
			Ok(duration) => duration.as_millis(),
			Err(error) => panic!("Clock may have gone backwards: {:?}", error.duration()),
		};
		Timestamp {
			value: now as u64,
		}
	}
}

impl Default for SystemClock {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	use crate::kvs::clock::SystemClock;

	#[test]
	fn get_clock_now() {
		let clock = SystemClock::new();
		let _ = clock.now();
	}
}
