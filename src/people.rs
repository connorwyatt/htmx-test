use std::{str::FromStr, sync::{Arc, RwLock}, time::Duration};

use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{FromRef, Path, State}, http::StatusCode, routing::{delete, get}, Form, Router
};
use chrono::NaiveDate;
use serde::Deserialize;
use tokio::time::sleep;
use uuid::Uuid;

use crate::AppState;

pub(crate) fn people_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_people).post(add_person))
        .route("/:id", delete(delete_person))
}

#[derive(Clone)]
pub(crate) struct PeopleState {
    people: Vec<Person>,
}

impl Default for PeopleState {
    fn default() -> Self {
        Self {
            people: vec![
                Person {
                    id: Uuid::new_v4().to_string(),
                    name: "John Doe".to_string(),
                    date_of_birth: NaiveDate::from_str("1984-01-01").unwrap(),
                    nationality: "GB".to_string(),
                },
                Person {
                    id: Uuid::new_v4().to_string(),
                    name: "Frankie Smith".to_string(),
                    date_of_birth: NaiveDate::from_str("1963-12-27").unwrap(),
                    nationality: "US".to_string(),
                },
            ],
        }
    }
}

impl PeopleState {
    fn get_people(&self) -> &Vec<Person> {
        &self.people
    }

    fn get_person(&self, person_id: &str) -> Option<&Person> {
        self.people.iter().find(|person| person.id == person_id)
    }

    fn add_person(&mut self, person: Person) {
        self.people.insert(0, person)
    }

    fn delete_person(&mut self, person_id: String) {
        self.people.retain(|person| person.id != person_id)
    }
}

impl FromRef<AppState> for Arc<RwLock<PeopleState>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.people_state.clone()
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
    sleep(Duration::from_secs(3)).await;

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

async fn delete_person(
    State(people_state): State<Arc<RwLock<PeopleState>>>,
    Path(person_id): Path<String>,
) -> impl IntoResponse {
    sleep(Duration::from_secs(3)).await;

    {
        let people_state_read = people_state.read().unwrap();
        let person = people_state_read.get_person(&person_id);

        if person.is_none() {
            return StatusCode::NOT_FOUND;
        }
    }

    let mut people_state_write = people_state.write().unwrap();
    people_state_write.delete_person(person_id);

    StatusCode::OK
}

#[derive(Template)]
#[template(path = "people.html")]
struct PeopleTemplate {
    people: Vec<Person>,
    form_state: AddPersonFormState,
}

#[derive(Template)]
#[template(path = "add_person_response.html")]
struct AddPersonResponseTemplate {
    person: Option<Person>,
    form_state: AddPersonFormState,
}
