use futures::TryStreamExt;
use mongodb::{bson::{doc, oid::ObjectId, Document}, options::{ClientOptions, FindOneAndUpdateOptions, FindOptions, ReturnDocument, ResolverConfig}, Client, Database};
use serde_json::{json, Value};
use std::env;
use tokio::sync::OnceCell;

static DB: OnceCell<Database> = OnceCell::const_new();

pub async fn database() -> Database {
    DB.get_or_init(|| async {
        let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let mut options = ClientOptions::parse_with_resolver_config(&uri, ResolverConfig::cloudflare())
            .await
            .expect("failed to parse MongoDB URI");
        options.app_name = Some("omnistore".to_string());

        let client = Client::with_options(options).expect("failed to create MongoDB client");
        client.database("omnistore")
    })
    .await
    .clone()
}

pub fn parse_object_id(id: &str) -> Option<ObjectId> {
    ObjectId::parse_str(id).ok()
}

pub fn doc_to_json(doc: Document) -> Value {
    serde_json::to_value(doc).unwrap_or_else(|_| json!(null))
}

pub async fn find_one_by_filter(collection: &str, filter: Document) -> Result<Option<Value>, String> {
    let coll = database().await.collection::<Document>(collection);
    coll.find_one(filter, None).await.map_err(|e| e.to_string()).map(|opt| opt.map(doc_to_json))
}

pub async fn find_all(collection: &str, filter: Document) -> Result<Vec<Value>, String> {
    let coll = database().await.collection::<Document>(collection);
    let mut cursor = coll.find(filter, None).await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
        results.push(doc_to_json(doc));
    }

    Ok(results)
}

pub async fn find_one(collection: &str, id: &str) -> Result<Option<Value>, String> {
    let coll = database().await.collection::<Document>(collection);
    let object_id = parse_object_id(id).ok_or_else(|| "invalid ObjectId".to_string())?;
    let filter = doc! {"_id": object_id};
    coll.find_one(filter, None).await.map_err(|e| e.to_string()).map(|opt| opt.map(doc_to_json))
}

pub async fn insert_one(collection: &str, document: Document) -> Result<Value, String> {
    let coll = database().await.collection::<Document>(collection);
    let result = coll.insert_one(document, None).await.map_err(|e| e.to_string())?;
    Ok(json!({"inserted_id": result.inserted_id}))
}

pub async fn update_one(collection: &str, id: &str, updates: Document) -> Result<Option<Value>, String> {
    let coll = database().await.collection::<Document>(collection);
    let object_id = parse_object_id(id).ok_or_else(|| "invalid ObjectId".to_string())?;
    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();

    coll.find_one_and_update(
        doc! {"_id": object_id},
        doc! {"$set": updates},
        options,
    )
    .await
    .map_err(|e| e.to_string())
    .map(|opt| opt.map(doc_to_json))
}

pub async fn delete_one(collection: &str, id: &str) -> Result<u64, String> {
    let coll = database().await.collection::<Document>(collection);
    let object_id = parse_object_id(id).ok_or_else(|| "invalid ObjectId".to_string())?;
    coll.delete_one(doc! {"_id": object_id}, None)
        .await
        .map_err(|e| e.to_string())
        .map(|result| result.deleted_count)
}

pub async fn ping() -> bool {
    database().await.run_command(doc! {"ping": 1}, None).await.is_ok()
}

pub async fn find_one_and_update_by_filter(
    collection: &str,
    filter: Document,
    update: Document,
) -> Result<Option<Value>, String> {
    let coll = database().await.collection::<Document>(collection);
    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();
    coll.find_one_and_update(filter, update, options)
        .await
        .map_err(|e| e.to_string())
        .map(|opt| opt.map(doc_to_json))
}

pub async fn find_paginated(
    collection: &str,
    filter: Document,
    page: u64,
    limit: u64,
) -> Result<(Vec<Value>, u64), String> {
    let coll = database().await.collection::<Document>(collection);
    let skip = page.saturating_sub(1) * limit;
    let total = coll
        .count_documents(filter.clone(), None)
        .await
        .map_err(|e| e.to_string())?;
    let options = FindOptions::builder()
        .skip(skip)
        .limit(limit as i64)
        .build();
    let mut cursor = coll.find(filter, options).await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
        results.push(doc_to_json(doc));
    }
    Ok((results, total))
}

pub async fn find_by_ids_paginated(
    collection: &str,
    ids: Vec<ObjectId>,
    page: u64,
    limit: u64,
) -> Result<(Vec<Value>, u64), String> {
    let filter = doc! { "_id": { "$in": ids } };
    find_paginated(collection, filter, page, limit).await
}
