use askama_axum::Template;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, RwLock},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Form, Router};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router()).await.unwrap();
}

fn router() -> Router {
    let serve_dir = ServeDir::new("assets");
    Router::new()
        .route("/", get(index))
        .route("/current_datetime", get(current_datetime))
        .route("/current_datetime_block", get(current_datetime_block))
        .route("/people", get(get_people).post(add_person))
        .route("/people/table-body", get(get_people_table_body))
        .fallback_service(serve_dir)
        .with_state(Arc::new(RwLock::new(PeopleState::default())))
}

// Home

async fn index() -> impl IntoResponse {
    IndexTemplate
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

// Current Datetime

async fn current_datetime() -> impl IntoResponse {
    CurrentDateTimeTemplate {
        datetime: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

async fn current_datetime_block() -> impl IntoResponse {
    CurrentDateTimeBlockTemplate {
        datetime: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

#[derive(Template)]
#[template(path = "current_datetime.html")]
struct CurrentDateTimeTemplate {
    datetime: String,
}

#[derive(Template)]
#[template(path = "current_datetime_block.html")]
struct CurrentDateTimeBlockTemplate {
    datetime: String,
}

// People

#[derive(Default)]
struct PeopleState {
    people: Vec<Person>,
}

impl PeopleState {
    fn get_people(&self) -> &Vec<Person> {
        &self.people
    }

    fn add_person(&mut self, person: Person) {
        self.people.insert(0, person)
    }
}

#[derive(Clone)]
struct Person {
    id: String,
    name: String,
    date_of_birth: NaiveDate,
    nationality: String,
}

#[derive(Default)]
struct AddPersonFormState {
    values: AddPersonFormValues,
    errors: AddPersonFormErrors,
}

#[derive(Clone, Default, Deserialize)]
struct AddPersonFormValues {
    name: String,
    date_of_birth: String,
    nationality: String,
}

impl TryInto<Person> for AddPersonFormValues {
    type Error = AddPersonFormErrors;

    fn try_into(self) -> Result<Person, Self::Error> {
        let date_of_birth = NaiveDate::from_str(&self.date_of_birth).map_err(|_| Self::Error {
            name: None,
            date_of_birth: Some(String::from(
                "The date of birth is not valid, should be in format YYYY-MM-DD.",
            )),
            nationality: None,
        })?;
        Ok(Person {
            id: Uuid::new_v4().to_string(),
            name: self.name,
            date_of_birth,
            nationality: self.nationality,
        })
    }
}

#[derive(Debug, Default)]
struct AddPersonFormErrors {
    name: Option<String>,
    date_of_birth: Option<String>,
    nationality: Option<String>,
}

async fn get_people(State(people_state): State<Arc<RwLock<PeopleState>>>) -> impl IntoResponse {
    let people_state_read = people_state.read().unwrap();
    PeopleTemplate {
        people: people_state_read.get_people().to_vec(),
        form_state: AddPersonFormState::default(),
    }
}

async fn add_person(
    State(people_state): State<Arc<RwLock<PeopleState>>>,
    Form(input): Form<AddPersonFormValues>,
) -> impl IntoResponse {
    let person: Person = match input.clone().try_into() {
        Ok(person) => person,
        Err(errors) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                AddPersonResponseTemplate {
                    person: None,
                    form_state: AddPersonFormState {
                        values: input,
                        errors,
                    },
                },
            )
        }
    };

    {
        let mut people_state_write = people_state.write().unwrap();
        people_state_write.add_person(person.clone());
    }

    (
        StatusCode::OK,
        AddPersonResponseTemplate {
            person: Some(person),
            form_state: AddPersonFormState::default(),
        },
    )
}

async fn get_people_table_body(
    State(people_state): State<Arc<RwLock<PeopleState>>>,
) -> impl IntoResponse {
    let people_state_read = people_state.read().unwrap();
    PeopleTableBodyBlockTemplate {
        people: people_state_read.get_people().to_vec(),
    }
}

#[derive(Template)]
#[template(path = "people.html")]
struct PeopleTemplate {
    people: Vec<Person>,
    form_state: AddPersonFormState,
}

#[derive(Template)]
#[template(path = "people_table_body_block.html")]
struct PeopleTableBodyBlockTemplate {
    people: Vec<Person>,
}

#[derive(Template)]
#[template(path = "add_person_form_block.html")]
struct AddPersonFormBlockTemplate {
    form_state: AddPersonFormState,
}

#[derive(Template)]
#[template(path = "add_person_response.html")]
struct AddPersonResponseTemplate {
    person: Option<Person>,
    form_state: AddPersonFormState,
}
