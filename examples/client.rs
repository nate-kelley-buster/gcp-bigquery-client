use gcp_bigquery_client::error::BQError;
use gcp_bigquery_client::model::dataset::Dataset;
use gcp_bigquery_client::model::query_request::QueryRequest;
use gcp_bigquery_client::model::table::Table;
use gcp_bigquery_client::model::table_data_insert_all_request::TableDataInsertAllRequest;
use gcp_bigquery_client::model::table_field_schema::TableFieldSchema;
use gcp_bigquery_client::model::table_schema::TableSchema;
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct MyRow {
    int_value: i64,
    float_value: f64,
    bool_value: bool,
    string_value: String,
    record_value: FirstRecordLevel,
}

#[derive(Serialize)]
struct FirstRecordLevel {
    int_value: i64,
    string_value: String,
    record_value: SecondRecordLevel,
}

#[derive(Serialize)]
struct SecondRecordLevel {
    int_value: i64,
    string_value: String,
}

#[tokio::main]
async fn main() -> Result<(), BQError> {
    // Read configuration parameters from environment variables
    let (ref project_id, ref dataset_id, ref table_id, ref gcp_sa_key) = env_vars();

    // Init BigQuery client
    let client = gcp_bigquery_client::Client::new(gcp_sa_key).await;

    // Create dataset
    let created_dataset = client.dataset().create(project_id, Dataset::new(dataset_id)).await?;
    println!(
        "Dataset '{}.{}' created",
        created_dataset.project_id(),
        created_dataset.dataset_id()
    );

    // Create table schema
    let table = Table::new(
        project_id,
        dataset_id,
        table_id,
        TableSchema::new(vec![
            TableFieldSchema::integer("int_value"),
            TableFieldSchema::float("float_value"),
            TableFieldSchema::bool("bool_value"),
            TableFieldSchema::string("string_value"),
            TableFieldSchema::record(
                "record_value",
                vec![
                    TableFieldSchema::integer("int_value"),
                    TableFieldSchema::string("string_value"),
                    TableFieldSchema::record(
                        "record_value",
                        vec![
                            TableFieldSchema::integer("int_value"),
                            TableFieldSchema::string("string_value"),
                        ],
                    ),
                ],
            ),
        ]),
    );

    let created_table = client.table().create(project_id, dataset_id, table).await?;
    println!(
        "Table '{}.{}.{}' created",
        created_table.project_id(),
        created_table.dataset_id(),
        created_table.table_id()
    );

    // Insert data via BigQuery Streaming API
    let mut insert_request = TableDataInsertAllRequest::new();
    insert_request.add_row(
        None,
        MyRow {
            int_value: 1,
            float_value: 1.0,
            bool_value: false,
            string_value: "first".into(),
            record_value: FirstRecordLevel {
                int_value: 10,
                string_value: "sub_level_1.1".into(),
                record_value: SecondRecordLevel {
                    int_value: 20,
                    string_value: "leaf".to_string(),
                },
            },
        },
    )?;
    insert_request.add_row(
        None,
        MyRow {
            int_value: 2,
            float_value: 2.0,
            bool_value: true,
            string_value: "second".into(),
            record_value: FirstRecordLevel {
                int_value: 11,
                string_value: "sub_level_1.2".into(),
                record_value: SecondRecordLevel {
                    int_value: 21,
                    string_value: "leaf".to_string(),
                },
            },
        },
    )?;
    insert_request.add_row(
        None,
        MyRow {
            int_value: 3,
            float_value: 3.0,
            bool_value: false,
            string_value: "third".into(),
            record_value: FirstRecordLevel {
                int_value: 12,
                string_value: "sub_level_1.3".into(),
                record_value: SecondRecordLevel {
                    int_value: 22,
                    string_value: "leaf".to_string(),
                },
            },
        },
    )?;
    insert_request.add_row(
        None,
        MyRow {
            int_value: 4,
            float_value: 4.0,
            bool_value: true,
            string_value: "fourth".into(),
            record_value: FirstRecordLevel {
                int_value: 13,
                string_value: "sub_level_1.4".into(),
                record_value: SecondRecordLevel {
                    int_value: 23,
                    string_value: "leaf".to_string(),
                },
            },
        },
    )?;

    client
        .tabledata()
        .insert_all(project_id, dataset_id, table_id, insert_request)
        .await?;

    // Query
    let mut rs = client
        .job()
        .query(
            project_id,
            QueryRequest::new(format!(
                "SELECT COUNT(*) AS c FROM `{}.{}.{}`",
                project_id, dataset_id, table_id
            )),
        )
        .await?;
    while rs.next_row() {
        println!("Number of rows inserted: {}", rs.get_i64_by_name("c")?.unwrap());
    }

    // Delete the table previously created
    client.table().delete(project_id, dataset_id, table_id).await?;

    // Delete the dataset previously created
    client.dataset().delete(project_id, dataset_id, true).await?;

    Ok(())
}

pub fn env_vars() -> (String, String, String, String) {
    let project_id = env::var("PROJECT_ID").expect("PROJECT_ID env var not defined");
    let dataset_id = env::var("DATASET_ID").expect("DATASET_ID env var not defined");
    let table_id = env::var("TABLE_ID").expect("TABLE_ID env var not defined");
    let gcp_sa_key =
        env::var("GOOGLE_APPLICATION_CREDENTIALS").expect("GOOGLE_APPLICATION_CREDENTIALS env var not defined");

    (project_id, dataset_id, table_id, gcp_sa_key)
}
