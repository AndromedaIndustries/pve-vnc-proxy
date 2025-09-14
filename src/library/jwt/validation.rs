use supabase_jwt::{Claims, JwksCache};
use tracing;

pub async fn validator(bearer_token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let jwks_url = std::env::var("SUPABASE_JWKS_URL")?;

    let jwks_cache = JwksCache::new(&jwks_url);

    tracing::debug!(jwks_url);

    match Claims::from_bearer_token(bearer_token, &jwks_cache).await {
        Ok(claims) => {
            tracing::debug!(
                "Successfully validated token for user: {}",
                claims.user_id()
            );
            Ok(claims)
        }
        Err(auth_error) => {
            tracing::error!("Claim Invalid");
            tracing::error!("{}", &auth_error);
            Err(Box::new(auth_error))
        }
    }
}
