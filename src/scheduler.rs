use crate::db::{JobType, Schedule, DB};
use sqlx::types::chrono::{self, NaiveDateTime};
use std::{
	str::FromStr,
	sync::{
		atomic::{AtomicBool, Ordering},
		mpsc, Arc,
	},
	thread,
};

pub struct Scheduler<'a> {
	db: &'a DB,
	rx: mpsc::Receiver<ThreadMessage>,
}

impl<'a> Scheduler<'a> {
	pub async fn new(db: &'a DB) -> Result<Scheduler<'a>, String> {
		let schedules: Vec<Schedule> = match db.get_schedules().await {
			Ok(schedules) => schedules,
			Err(err) => return Err(format!("Failed to get schedules\n{}", err)),
		};

		for schedule in schedules {
			// Make sure there is a job for each schedule
			match db.get_job(schedule.id as u64).await {
				Ok(_) => (),
				Err(err) => match err {
					// If there isn't, create one
					sqlx::Error::RowNotFound => {
						let run_at: NaiveDateTime = get_next_run_datetime(&schedule.cron);
						match db
							.create_job(JobType::Monitoring, run_at, Option::from(schedule.id))
							.await
						{
							Ok(_) => (),
							Err(err) => return Err(format!("Failed to create job\n{}", err)),
						}
					}
					_ => return Err(format!("Failed to get job\n{}", err)),
				},
			}
		}

		let (_, rx): (mpsc::Sender<ThreadMessage>, mpsc::Receiver<ThreadMessage>) = mpsc::channel();
		let this: Scheduler = Scheduler { db, rx };
		this.create_thread();

		Ok(this)
	}

	/// Refresh when the next job should run
	pub async fn refresh() {}

	/// Create a thread to run the scheduler
	fn create_thread(&self) {
		// 	let receiver: &mpsc::Receiver<ThreadMessage> = &self.rx;
		// 	let db: Arc<&DB> = Arc::new(self.db);

		// 	thread::spawn(move || {
		// 		loop {
		// 			match receiver.try_recv() {
		// 				Ok(msg) => match msg {
		// 					ThreadMessage::Refresh => (),
		// 				},
		// 				Err(mpsc::TryRecvError::Empty) => (),
		// 				Err(mpsc::TryRecvError::Disconnected) => panic!("Scheduler thread disconnected"),
		// 			}
		// 		}
		// 	});
		// }
		println!("Not implemented");
	}
}

fn get_next_run_datetime(exp: &str) -> NaiveDateTime {
	let schedule: cron::Schedule = cron::Schedule::from_str(exp).unwrap();
	let next: chrono::DateTime<chrono::Utc> = schedule.upcoming(chrono::Utc).next().unwrap();
	next.naive_utc()
}

enum ThreadMessage {
	Refresh,
}
