use aws_config::Region;
use aws_sdk_s3::{
    config::Credentials,
    presigning::PresigningConfig,
    types::{
        AbortIncompleteMultipartUpload, BucketLifecycleConfiguration, ExpirationStatus,
        LifecycleExpiration, LifecycleRule, LifecycleRuleFilter,
    },
    Client,
};
use std::{env, time::Duration};
use tokio::sync::OnceCell;

const BUCKET: &str = "omnistore";
const TEMP_PREFIX: &str = "temp/";
const PRESIGN_TTL_SECS: u64 = 900; // 15 minutos

static S3: OnceCell<Client> = OnceCell::const_new();

pub async fn client() -> Client {
    S3.get_or_init(|| async {
        let endpoint = env::var("MINIO_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());
        let access_key = env::var("MINIO_ACCESS_KEY")
            .unwrap_or_else(|_| "minioadmin".to_string());
        let secret_key = env::var("MINIO_SECRET_KEY")
            .unwrap_or_else(|_| "minioadmin".to_string());

        let credentials = Credentials::new(access_key, secret_key, None, None, "minio");
        let config = aws_config::from_env()
            .region(Region::new("us-east-1"))
            .credentials_provider(credentials)
            .endpoint_url(&endpoint)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true) // MinIO exige path-style
            .build();

        Client::from_conf(s3_config)
    })
    .await
    .clone()
}

/// Garante que o bucket existe e aplica lifecycle policies (idempotente).
pub async fn ensure_bucket() {
    let s3 = client().await;

    // Cria o bucket se não existir
    let _ = s3
        .create_bucket()
        .bucket(BUCKET)
        .send()
        .await;

    // Lifecycle: expirar temp/ em 1 dia + abort incomplete multipart em 1 dia
    let expiration = LifecycleExpiration::builder().days(1).build();
    let abort = AbortIncompleteMultipartUpload::builder()
        .days_after_initiation(1)
        .build();
    let filter = LifecycleRuleFilter::builder()
        .prefix(TEMP_PREFIX)
        .build();
    let rule = LifecycleRule::builder()
        .id("temp-cleanup")
        .status(ExpirationStatus::Enabled)
        .filter(filter)
        .expiration(expiration)
        .abort_incomplete_multipart_upload(abort)
        .build()
        .expect("lifecycle rule");

    let lifecycle = BucketLifecycleConfiguration::builder()
        .rules(rule)
        .build()
        .expect("lifecycle config");

    let _ = s3
        .put_bucket_lifecycle_configuration()
        .bucket(BUCKET)
        .lifecycle_configuration(lifecycle)
        .send()
        .await;
}

/// Gera uma Presigned URL para o frontend fazer PUT direto no MinIO.
/// Todos os uploads iniciais vão para temp/{uuid}-{filename}.
/// Retorna (key, presigned_url).
pub async fn presigned_upload_url(filename: &str) -> Result<(String, String), String> {
    let s3 = client().await;
    let uuid = uuid::Uuid::new_v4();
    let key = format!("{TEMP_PREFIX}{uuid}-{filename}");

    let presigning = PresigningConfig::expires_in(Duration::from_secs(PRESIGN_TTL_SECS))
        .map_err(|e| e.to_string())?;

    let presigned = s3
        .put_object()
        .bucket(BUCKET)
        .key(&key)
        .presigned(presigning)
        .await
        .map_err(|e| e.to_string())?;

    Ok((key, presigned.uri().to_string()))
}

/// Move objeto de temp/{key} para {dest_folder}/{filename} via CopyObject + Delete.
/// Retorna a chave final.
pub async fn confirm_upload(temp_key: &str, dest_folder: &str) -> Result<String, String> {
    let s3 = client().await;

    let filename = temp_key
        .rsplit_once('-')
        .map(|(_, f)| f)
        .unwrap_or(temp_key);

    let dest_key = format!("{dest_folder}/{filename}");
    let copy_source = format!("{BUCKET}/{temp_key}");

    s3.copy_object()
        .bucket(BUCKET)
        .copy_source(&copy_source)
        .key(&dest_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    s3.delete_object()
        .bucket(BUCKET)
        .key(temp_key)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(dest_key)
}

/// Deleta um objeto do bucket (ex: ao remover produto).
pub async fn delete_object(key: &str) -> Result<(), String> {
    client()
        .await
        .delete_object()
        .bucket(BUCKET)
        .key(key)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Deleta múltiplos objetos de uma vez (ex: ao remover produto com várias fotos).
pub async fn delete_objects(keys: Vec<String>) -> Result<u32, String> {
    use aws_sdk_s3::types::{Delete, ObjectIdentifier};

    if keys.is_empty() {
        return Ok(0);
    }

    let objects: Result<Vec<_>, _> = keys
        .iter()
        .map(|k| ObjectIdentifier::builder().key(k).build().map_err(|e| e.to_string()))
        .collect();

    let delete = Delete::builder()
        .set_objects(Some(objects?))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client()
        .await
        .delete_objects()
        .bucket(BUCKET)
        .delete(delete)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(resp.deleted().len() as u32)
}

/// URL pública para um objeto permanente.
pub fn public_url(key: &str) -> String {
    let endpoint = env::var("MINIO_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());
    format!("{endpoint}/{BUCKET}/{key}")
}
