use sqlx::{
	migrate, query_as,
	types::chrono::NaiveDateTime,
	Error, MySql, MySqlPool, Pool, query, mysql::MySqlQueryResult,
};
use std::env;

pub struct DB {
	pub pool: MySqlPool,
}

impl DB {
	pub async fn new() -> Result<DB, String> {
		let url: String =
			env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");

		match url.split('/').last() {
			Some(_) => (),
			None => {
				return Err(String::from(
					"DATABASE_URL environment variable does not contain a database name",
				))
			}
		}

		let pool: Pool<MySql> = match MySqlPool::connect(&url).await {
			Ok(pool) => pool,
			Err(err) => return Err(format!("Failed to connect to database\n{}", err)),
		};

		let db: DB = DB { pool };
		match migrate!().run(&db.pool).await {
			Ok(_) => Ok(db),
			Err(err) => Err(format!("Failed to migrate database\n{}", err)),
		}
	}

	pub async fn get_user(&self, id: i32) -> Result<User, Error> {
		query_as!(User, "SELECT id, name FROM users WHERE id = ?", id)
			.fetch_one(&self.pool)
			.await
	}

	pub async fn get_schedules(&self) -> Result<Vec<Schedule>, Error> {
		query_as!(
			Schedule,
			"SELECT id, name, cron, enabled as 'enabled: bool', created_at FROM schedules"
		)
		.fetch_all(&self.pool)
		.await
	}

	pub async fn get_job(&self, id: u64) -> Result<Job, Error> {
		query_as!(Job, "SELECT id, type as 'job_type: JobType', linked_id, run_at FROM jobs WHERE id = ?", id)
			.fetch_one(&self.pool)
			.await
	}

	pub async fn get_next_job(&self) -> Result<Job, Error> {
		query_as!(Job, "SELECT id, type as 'job_type: JobType', linked_id, run_at FROM jobs ORDER BY run_at ASC LIMIT 1")
			.fetch_one(&self.pool)
			.await
	}

	pub async fn create_job(&self, job_type: JobType, run_at: NaiveDateTime, linked_id: Option<i32>) -> Result<(), Error> {
		let result: Result<MySqlQueryResult, Error> = match linked_id {
			Some(linked_id) => query!("INSERT INTO jobs (type, run_at, linked_id) VALUES (?, ?, ?)", job_type as i32, run_at, linked_id)
				.execute(&self.pool)
				.await,
			None => query!("INSERT INTO jobs (type, run_at) VALUES (?, ?)", job_type as i32, run_at)
				.execute(&self.pool)
				.await,
		};

		match result {
			Ok(_) => Ok(()),
			Err(err) => Err(err),
		}
	}
}

pub struct User {
	pub id: i32,
	pub name: String,
}

pub struct Schedule {
	pub id: i32,
	pub name: String,
	pub cron: String,
	pub enabled: bool,
	pub created_at: NaiveDateTime,
}

pub struct Job {
	pub id: u64,
	pub job_type: JobType,
	pub linked_id: Option<i32>,
	pub run_at: NaiveDateTime,
}

#[derive(sqlx::Type)]
#[repr(i32)]
pub enum JobType {
	Monitoring = 1,
}