use reqwest;
use serde::{Deserialize, Serialize};
use dotenvy;
use crate::AppError;

const IGDB_CLIENT_ID: &str = env!("IGDB_CLIENT_ID");
const IGDB_CLIENT_SECRET: &str = env!("IGDB_CLIENT_SECRET");

#[derive(Serialize)]
pub struct TwitchTokenRequest
{
    client_id: String,
    client_secret: String,
    grant_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct TwitchTokenResponse
{
    access_token: String,
    expires_in: i64,
    token_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct GameCover
{
    pub cover: Option<Cover>,
}

#[derive(Serialize, Deserialize)]
pub struct Cover
{
    pub url: String,
}

impl Cover
{
    pub fn format_url(&mut self) 
    {
        if self.url.starts_with("//") 
        {
            self.url = format!("https:{}", self.url);
        }

        self.url = self.url.replace("t_thumb", "t_cover_big");
    }
}

pub async fn get_access_token() -> Result<(TwitchTokenResponse, TwitchTokenRequest), AppError>
{
    //comment to force recompile again. again
    let _ = dotenvy::dotenv().ok();

    let client = reqwest::Client::new();

    let request_url = "https://id.twitch.tv/oauth2/token";

    let params = TwitchTokenRequest 
    {
        client_id: IGDB_CLIENT_ID.to_string(),
        client_secret: IGDB_CLIENT_SECRET.to_string(),
        grant_type: "client_credentials".to_string()
    };


    let response = client.post(request_url)
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() 
    {
        let raw_error = response.text().await?;
        return Err(AppError::Message(format!("API Error: {}", raw_error))); 
    }

    let data = response.json::<TwitchTokenResponse>().await?;

    Ok((data, params))
}

pub async fn get_cover_art(name: &str, quick_fetch: bool) -> Result<Vec<GameCover>, AppError>
{
    let token = get_access_token().await?;

    let client = reqwest::Client::new();

    let request_url = "https://api.igdb.com/v4/games";

    let limit = if quick_fetch {1} else {10};

    let response = client.post(request_url)
        .header("Accept", "application/json")
        .header("Client-ID", token.1.client_id)
        .header("Authorization", format!("Bearer {}", token.0.access_token))
        .body(format!("search \"{}\"; fields name,cover.url; limit {};", name, limit))
        .send()
        .await?;

    let mut data = response.json::<Vec<GameCover>>().await?;

    for entry in &mut data
    {
        if let Some(ref mut cover) = entry.cover
        {
            cover.format_url();
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests
{
    // API tests are skipped to prevent excessive API usage.

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_quick_fetch()
    {
        let result = get_cover_art("Umineko: When They Cry", true).await;

        let result_success = result.expect("The API request failed!");

        for entry in result_success
        {
            match entry.cover
            {
                Some(cover) => print!("Cover: {} ,", cover.url),
                None => print!("Cover: None "),
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_art_list()
    {
        let result = get_cover_art("Umineko: When They Cry", false).await;

        let result_success = result.expect("The API request failed!");

        for entry in result_success
        {
            match entry.cover
            {
                Some(cover) => print!("Cover: {} ,", cover.url),
                None => print!("Cover: None "),
            }
        }
    }

    #[test]
    #[ignore]
    fn test_cover_format_url_with_slashes() 
    {
        let mut cover = Cover {
            url: "//images.igdb.com/igdb/image/upload/t_thumb/co2x8j.jpg".to_string(),
        };
        cover.format_url();
        
        // Should prepend "https:" and replace "t_thumb" with "t_cover_big"
        assert_eq!(
            cover.url,
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co2x8j.jpg"
        );
    }

    #[test]
    #[ignore]
    fn test_cover_format_url_without_slashes() 
    {
        let mut cover = Cover {
            url: "https://images.igdb.com/igdb/image/upload/t_thumb/co2x8j.jpg".to_string(),
        };
        cover.format_url();
        
        // Should leave the prefix alone since it doesn't start with "//", but still replace "t_thumb"
        assert_eq!(
            cover.url,
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co2x8j.jpg"
        );
    }
}