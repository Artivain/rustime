use crate::{db::{Job, JobType, Method, Schedule, DB}, USER_AGENT};
use reqwest::{Client, ClientBuilder};
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};
use std::{str::FromStr, time::Duration};

pub struct Scheduler {}

impl Scheduler {
	pub async fn new() -> Result<Scheduler, String> {
		let db: DB = match DB::new().await {
			Ok(db) => db,
			Err(err) => return Err(err),
		};

		// Get all schedules from database
		let schedules: Vec<Schedule> = match db.get_schedules().await {
			Ok(schedules) => schedules,
			Err(err) => return Err(format!("Failed to get schedules from database\n{}", err)),
		};

		// Make sure they will all run
		for schedule in schedules {
			if schedule.enabled {
				// If the schedule is enabled, make sure it has a job
				match db.get_job_by_linked_id(schedule.id).await {
					Ok(_) => (),
					Err(_) => {
						// Get next run time
						let run_at: NaiveDateTime = match get_next_run_time(&schedule.cron) {
							Some(next_run_time) => next_run_time,
							None => {
								println!(
									"Failed to get next run time for schedule {}",
									schedule.id
								);
								continue;
							}
						};

						// Create job
						db.create_job(JobType::Monitoring, run_at, Some(schedule.id))
							.await
							.unwrap();
					}
				};
			}
		}

		tokio::spawn(async {
			loop {
				tokio::spawn(async {
					println!("Checking for jobs to run");

					let db: DB = match DB::new().await {
						Ok(db) => db,
						Err(err) => {
							println!("Failed to connect to database\n{}", err);
							return;
						}
					};

					let client: Client = ClientBuilder::new()
						.timeout(Duration::from_secs(30))
						.user_agent(USER_AGENT)
						.build()
						.unwrap();

					let jobs: Vec<Job> = match db.get_jobs_to_run().await {
						Ok(jobs) => jobs,
						Err(err) => {
							println!("Failed to get jobs to run\n{}", err);
							return;
						}
					};

					for job in jobs {
						match db.delete_job(job.id).await {
							Ok(_) => (),
							Err(_) => continue,
						}

						match job.job_type {
							JobType::Monitoring => {
								println!("Running monitoring job {}", job.id);

								let schedule: Schedule =
									match db.get_schedule(job.linked_id.unwrap()).await {
										Ok(schedule) => schedule,
										Err(err) => {
											println!(
												"Failed to get schedule for job {}\n{}",
												job.id, err
											);
											continue;
										}
									};

								let (is_up, down_reason) = match schedule.method {
									Method::HEAD => {
										execute_http(
											&client,
											&schedule.target,
											reqwest::Method::HEAD,
										)
										.await
									}
									Method::GET => {
										execute_http(
											&client,
											&schedule.target,
											reqwest::Method::GET,
										)
										.await
									}
									Method::POST => {
										execute_http(
											&client,
											&schedule.target,
											reqwest::Method::POST,
										)
										.await
									}
								};

								if is_up != schedule.is_up {
									match is_up {
										true => {
											db.set_schedule_up(schedule.id).await.unwrap_or(());

										},
										false => db.set_schedule_down(
											schedule.id,
											down_reason.unwrap_or("Unknown".to_string()),
										),
									}
								} else if !is_up
									&& down_reason.unwrap() != schedule.down_reason.unwrap()
								{
									db.set_schedule_down(
										schedule.id,
										down_reason.unwrap_or("Unknown".to_string()),
									)
									.await
									.unwrap_or(());
								}
							}
						}
					}

					db.close().await;
				});

				tokio::time::sleep(Duration::from_secs(10)).await;
			}
		});

		db.close().await;
		Ok(Scheduler {})
	}
}

fn get_next_run_time(cron: &str) -> Option<NaiveDateTime> {
	let s: cron::Schedule = match cron::Schedule::from_str(cron) {
		Ok(s) => s,
		Err(_) => return None,
	};

	// Get the next run time
	let next: Option<DateTime<Utc>> = s.upcoming(Utc).next();

	match next {
		Some(next) => Some(next.naive_utc()),
		None => None,
	}
}

async fn execute_http(
	client: &Client,
	url: &str,
	method: reqwest::Method,
) -> (bool, Option<String>) {
	match client.request(method, url).build() {
		Ok(req) => match client.execute(req).await {
			Ok(res) => {
				if res.status().is_success() {
					(true, None)
				} else {
					(
						false,
						Some(format!(
							"{} {}",
							res.status().as_u16(),
							res.status().canonical_reason().unwrap()
						)),
					)
				}
			}
			Err(err) => (false, Some(format!("Could not execute request: {}", err))),
		},
		Err(err) => (false, Some(format!("Could not build request: {}", err))),
	}
}
