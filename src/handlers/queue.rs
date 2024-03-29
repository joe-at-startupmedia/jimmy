//! HTTP handlers for the `/queue` endpoints.

use actix_web::{web, HttpResponse, Responder,  ResponseError};
use log::{debug, error};

use crate::application::{RedisManager, file};
use crate::models::{job, queue, ApplicationState, OcyError};

/// Handle `GET /queue` requests to get a JSON list of all existing queues.
///
/// # Returns
///
/// * 200 - JSON response containing list of queue names.
pub async fn index(data: web::Data<ApplicationState>) -> impl Responder {
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::RedisConnection(err).error_response(),
    };

    match RedisManager::queue_names(&mut conn).await {
        Ok(queue_names) => HttpResponse::Ok().json(queue_names),
        Err(err) => {
            error!("Failed to fetch queue names: {}", err);
            err.error_response()
        }
    }
}

/// Handles `PUT /queue/{queue_name}` requests.
pub async fn create_or_update(
    path: web::Path<String>,
    json: web::Json<queue::Settings>,
    data: web::Data<ApplicationState>,
) -> impl Responder {
    let queue_name = path.into_inner();
    let queue_settings = json.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::RedisConnection(err).error_response(),
    };

    match RedisManager::create_or_update_queue(&mut conn, &queue_name, &queue_settings).await {
        Ok(true) => HttpResponse::Created()
            .append_header(("Location", format!("/queue/{}", queue_name)))
            .finish(),
        Ok(false) => HttpResponse::NoContent()
            .reason("Queue setting updated")
            .append_header(("Location", format!("/queue/{}", queue_name)))
            .finish(),
        Err(err @ OcyError::BadRequest(_)) => err.error_response(),
        Err(err) => {
            error!("[queue:{}] failed to create/update queue: {}", &queue_name, err);
            err.error_response()
        }
    }
}

pub async fn delete(path: web::Path<String>, data: web::Data<ApplicationState>) -> impl Responder {
    let queue_name = path.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::RedisConnection(err).error_response(),
    };

    match RedisManager::delete_queue(&mut conn, &queue_name).await {
        Ok(true) => HttpResponse::NoContent().reason("Queue deleted").finish(),
        Ok(false) => HttpResponse::NotFound().reason("Queue not found").finish(),
        Err(err @ OcyError::BadRequest(_)) => err.error_response(),
        Err(err) => {
            error!("[queue:{}] failed to delete queue: {}", &queue_name, err);
            err.error_response()
        }
    }
}

pub async fn settings(
    path: web::Path<String>,
    data: web::Data<ApplicationState>,
) -> impl Responder {
    let queue_name = path.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::RedisConnection(err).error_response(),
    };
    match RedisManager::queue_settings(&mut conn, &queue_name).await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(err @ OcyError::NoSuchQueue(_)) => err.error_response(),
        Err(err) => {
            error!(
                "[queue:{}] failed to fetch queue summary: {}",
                &queue_name, err
            );
            err.error_response()
        },
    }
}

pub async fn size(path: web::Path<String>, data: web::Data<ApplicationState>) -> impl Responder {
    let queue_name = path.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::RedisConnection(err).error_response(),
    };
    match RedisManager::queue_size(&mut conn, &queue_name).await {
        Ok(size) => HttpResponse::Ok().json(size),
        Err(err @ OcyError::NoSuchQueue(_)) => err.error_response(),
        Err(err) => {
            error!(
                "[queue:{}] failed to fetch queue size: {}",
                &queue_name, err
            );
            err.error_response()
        }
    }
}

pub async fn job_ids(path: web::Path<String>, data: web::Data<ApplicationState>) -> impl Responder {
    let queue_name = path.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::RedisConnection(err).error_response(),
    };
    match RedisManager::queue_job_ids(&mut conn, &queue_name).await {
        Ok(size) => HttpResponse::Ok().json(size),
        Err(err @ OcyError::NoSuchQueue(_)) => err.error_response(),
        Err(err) => {
            error!(
                "[queue:{}] failed to fetch queue size: {}",
                &queue_name, err
            );
            err.error_response()
        }
    }
}

pub async fn create_job(
    path: web::Path<String>,
    json: web::Json<job::CreateRequest>,
    data: web::Data<ApplicationState>,
) -> impl Responder {
    let queue_name = path.into_inner();
    let job_req = json.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::from(err).error_response(),
    };
    let job_write_res = file::write_job(&queue_name, &job_req).unwrap();

    match RedisManager::create_job(&mut conn, &queue_name, &job_req).await {
        Ok(job_id) => {
            let job_attempt = file::get_job(&queue_name, job_write_res.1);
            debug!("deleting job attempt {:?}", job_attempt);
            let _del = file::delete_job(&queue_name, job_write_res.1);
            HttpResponse::Created()
                .append_header(("Location", format!("/job/{}", job_id)))
                .json(job_id)
        },
        Err(err @ OcyError::NoSuchQueue(_) | err @ OcyError::BadRequest(_) ) => err.error_response(),
        Err(err) => {
            error!("[queue:{}] failed to create new job: {}", &queue_name, err);
            err.error_response()
        }
    }
}

pub async fn next_job(
    path: web::Path<String>,
    data: web::Data<ApplicationState>,
) -> impl Responder {
    let queue_name = path.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::from(err).error_response(),
    };

    match RedisManager::next_queued_job(&mut conn, &queue_name).await {
        Ok(Some(job)) => HttpResponse::Ok().json(job),
        Ok(None) => match &data.config.server.next_job_delay {
            Some(delay) if !delay.is_zero() => {
                tokio::time::sleep(delay.0).await;
                HttpResponse::NoContent().into()
            }
            _ => HttpResponse::NoContent().into(),
        },
        Err(err) => {
            error!("[queue:{}] failed to fetch next job: {}", &queue_name, err);
            err.error_response()
        }
    }
}

pub async fn fetch_job(
    path: web::Path<(String, u64)>,
    data: web::Data<ApplicationState>,
) -> impl Responder {
    let (queue_name, job_id) = path.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::from(err).error_response(),
    };
    match RedisManager::fetch_queued_job(&mut conn, &queue_name, job_id).await {
        Ok(Some(job)) => HttpResponse::Ok().json(job),
        Ok(None) => match &data.config.server.next_job_delay {
            Some(delay) if !delay.is_zero() => {
                tokio::time::sleep(delay.0).await;
                HttpResponse::NoContent().into()
            }
            _ => HttpResponse::NoContent().into(),
        },
        Err(err) => {
            error!("[queue:{}] failed to fetch job {}: {}", &queue_name, job_id, err);
            err.error_response()
        }
    }
}


pub async fn reattempt_job(
    path: web::Path<(String, i64)>,
    data: web::Data<ApplicationState>,
) -> impl Responder {
    let (queue_name, timestamp) = path.into_inner();
    let mut conn = match data.redis_conn_pool.get().await {
        Ok(conn) => conn,
        Err(err) => return OcyError::from(err).error_response(),
    }; 

    debug!("attempting to reattempt {:?} on {}", timestamp, &queue_name);

    match file::get_job(&queue_name, timestamp) {
        Ok(mut job_req) => {
            debug!("attempting to reattempt {:?} on {}", job_req, timestamp);
            //this will not work in the input value is not an object
            if let Some(serde_json::Value::Object(input)) = &mut job_req.input {
                input.extend([ // note: requires fairly recent (stable) Rust, otherwise arrays are not `IntoIterator`
                    ("attempted_on".to_owned(), timestamp.into()),
                ]);
            }
            match RedisManager::create_job(&mut conn, &queue_name, &job_req).await {
                Ok(job_id) => {
                    debug!("deleting job attempt {:?} on {}", job_req, timestamp);
                    let _del = file::delete_job(&queue_name, timestamp);
                    HttpResponse::Created()
                        .append_header(("Location", format!("/job/{}", job_id)))
                        .json(job_id)
                },
                Err(err) => {
                    error!("[queue:{}] failed to reattempt creating new job: {}", &queue_name, err);
                    err.error_response()
                }
            }
        },
        Err(err) => {
            error!("[queue:{}] failed to reattempt failed job creation: {}", &queue_name, err);
            err.error_response()
        }
    }
}

