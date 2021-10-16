use anyhow::{Result, Context};
use async_trait::async_trait;
use multer::{Field, Multipart};
use rocket::{Data, Request, data::{FromData, Limits, Outcome}, form::Errors, http::Status};
use tracing::debug;

pub struct Form<'v>(Vec<Field<'v>>);

#[async_trait]
impl<'r> FromData<'r> for Form<'r> {
    type Error = anyhow::Error;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let mut multipart = match Self::from_multipart(req, data).await {
            Ok(m) => m,
            Err(err) => return Outcome::Failure((Status::InternalServerError, err)),
        };

        let mut value = Vec::new();

        // Iterate over the fields, use `next_field()` to get the next field.
        while let Ok(field) = multipart.next_field().await {
            debug!("pushed field");
            if let Some(field) = field {
                value.push(field);
            }
        }

        Outcome::Success(Form(value))
    }
}

impl<'r> Form<'r> {
    pub fn into_inner(self) -> Vec<Field<'r>> {
        self.0
    }

    async fn from_multipart(req: &'r Request<'_>, data: Data<'r>) -> Result<Multipart<'r>> {
        let boundary = req.content_type()
            .ok_or(multer::Error::NoMultipart)?
            .param("boundary")
            .ok_or(multer::Error::NoBoundary)?;

        let form_limit = req.limits()
            .get("data-form")
            .unwrap_or(Limits::DATA_FORM);

        Ok(Multipart::with_reader(data.open(form_limit), boundary))
    }
}
