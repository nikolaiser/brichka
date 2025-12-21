use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListCatalogsResponse{
    pub catalogs: Vec<Catalog>
}

#[derive(Deserialize)]
pub struct Catalog {
    pub name: String,
}

pub async fn list_catalogs() -> Result<ListCatalogsResponse> {
    let response = crate::client::call_databricks_api::<ListCatalogsResponse>("get", "/api/2.1/unity-catalog/catalogs", None).await?;
    Ok(response)
}


#[derive(Deserialize)]
pub struct ListSchemasResponse{
    pub schemas: Vec<Schema>
}

#[derive(Deserialize)]
pub struct Schema {
    pub name: String,
}

pub async fn list_schemas(catalog_name: String) -> Result<ListSchemasResponse> {
    let path = format!("/api/2.1/unity-catalog/schemas?catalog_name={}", catalog_name);
    let response = crate::client::call_databricks_api::<ListSchemasResponse>("get", &path, None).await?;
    Ok(response)
}

#[derive(Deserialize)]
pub struct ListTablesResponse{
    pub tables: Vec<Table>
}

#[derive(Deserialize)]
pub struct Table {
    pub name: String,
}

pub async fn list_tables(catalog_name: String, schema_name: String) -> Result<ListTablesResponse> {
    let path = format!("/api/2.1/unity-catalog/tables?catalog_name={}&schema_name={}&omit_columns=true&omit_properties=true&omit_username=true", catalog_name, schema_name);
    let response = crate::client::call_databricks_api::<ListTablesResponse>("get", &path, None).await?;
    Ok(response)
}
