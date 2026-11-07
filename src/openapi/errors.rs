use std::collections::BTreeMap;

use serde_json::json;
use utoipa::{
    Modify,
    openapi::{
        ContentBuilder, OpenApi, Ref, RefOr, Response, ResponseBuilder, example::ExampleBuilder,
    },
};

use crate::errors::catalog::{
    CatalogEntry, bad_request, internal_server_error, service_unavailable, unauthorized,
};

// Errors documented per operation, keyed by the operation id set in the handler annotation
const OPERATION_ERRORS: &[(&str, &[CatalogEntry])] = &[
    (
        "auth_login",
        &[
            unauthorized::MISSING_AUTH_HEADERS,
            unauthorized::INVALID_CREDENTIALS,
            bad_request::INVALID_HEADER_VALUES,
            bad_request::INVALID_INPUT,
            internal_server_error::TOKEN_SIGNING_FAILED,
            internal_server_error::DATABASE_ERROR,
            internal_server_error::SERVER_ERROR,
        ],
    ),
    ("health", &[service_unavailable::DB_PROBE_FAILED]),
];

// Builds one response per http status, each with a named example per catalogue entry
fn build_error_responses(entries: &[CatalogEntry]) -> BTreeMap<String, RefOr<Response>> {
    // Groups the entries by http status
    let mut grouped: BTreeMap<u16, Vec<&CatalogEntry>> = BTreeMap::new();

    for entry in entries {
        grouped
            .entry(entry.code.status_code())
            .or_default()
            .push(entry);
    }

    grouped
        .into_iter()
        .map(|(status, group)| {
            // The example name is the short error text of the entry
            let examples = group.iter().map(|entry| {
                let example = ExampleBuilder::new()
                    .value(Some(json!({
                        "success": false,
                        "statusCode": entry.code.status_code(),
                        "code": entry.code.as_str(),
                        "message": entry.message,
                        "error": entry.error,
                    })))
                    .build();

                (entry.error.to_string(), example)
            });

            let content = ContentBuilder::new()
                .schema(Some(Ref::from_schema_name("ErrorResponse")))
                .examples_from_iter(examples)
                .build();

            // The description is the code name with spaces, like the Elysia version
            let response = ResponseBuilder::new()
                .description(group[0].code.as_str().replace('_', " "))
                .content("application/json", content)
                .build();

            (status.to_string(), RefOr::T(response))
        })
        .collect()
}

// Adds the error responses of OPERATION_ERRORS to the matching operations of the spec
pub struct ErrorExamples;

impl Modify for ErrorExamples {
    fn modify(&self, openapi: &mut OpenApi) {
        for path_item in openapi.paths.paths.values_mut() {
            let operations = [
                &mut path_item.get,
                &mut path_item.post,
                &mut path_item.put,
                &mut path_item.delete,
                &mut path_item.patch,
            ];

            for operation in operations.into_iter().flatten() {
                let Some(operation_id) = operation.operation_id.clone() else {
                    continue;
                };

                let Some((_, entries)) = OPERATION_ERRORS
                    .iter()
                    .find(|(name, _)| *name == operation_id)
                else {
                    continue;
                };

                operation
                    .responses
                    .responses
                    .extend(build_error_responses(entries));
            }
        }
    }
}
