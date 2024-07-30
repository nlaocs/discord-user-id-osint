use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tokio::{fs::{File, OpenOptions}, io::{self, AsyncReadExt, AsyncWriteExt}};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    token: String,
}

impl Config {
    async fn get() -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = File::open("config.json").await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Clan {
    pub identity_guild_id: String,
    pub identity_enabled: bool,
    pub tag: String,
    pub badge: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AvatarDecorationData {
    pub asset: String,
    pub sku_id: String,
    pub expires_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserData {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub discriminator: String,
    pub public_flags: u64,
    pub flags: u64,
    pub bot: Option<bool>,
    pub banner: Option<String>,
    pub accent_color: Option<u32>,
    pub global_name: Option<String>,
    pub avatar_decoration_data: Option<AvatarDecorationData>,
    pub banner_color: Option<String>,
    pub clan: Option<Clan>,
}

#[derive(Eq, PartialEq)]
enum ImageType {
    Avatar,
    Banner,
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImageType::Avatar => write!(f, "avatars"),
            ImageType::Banner => write!(f, "banners"),
        }
    }
}

impl UserData {
    async fn get(token: &str, user_id: &str) -> Result<UserData, Box<dyn std::error::Error>> {
        let client = Client::new();
        let mut header = HeaderMap::new();
        header.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_value = format!("Bot {}", token);
        header.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
        let res = client.get(&format!("https://discord.com/api/v10/users/{}", user_id))
            .headers(header)
            .send().await?;
        let body = res.text().await?;
        let user_data: UserData = serde_json::from_str(&body)?;
        Ok(user_data)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::get().await?;
    let id = "";
    let user_data = UserData::get(&config.token, id).await?;
    println!("id: {}", user_data.id);
    println!("username: {}", user_data.username);
    println!("avatar: {}", user_data.avatar.unwrap_or_else(|| "None".to_string()));
    println!("discriminator: {}", user_data.discriminator);
    println!("public_flags: {}", user_data.public_flags);
    println!("flags: {}", user_data.flags);
    println!("bot: {}", user_data.bot.unwrap_or_else(|| false));
    println!("banner: {}", user_data.banner.unwrap_or_else(|| "None".to_string()));
    if user_data.accent_color.is_some() {
        println!("accent_color: {}", format!("#{:06x}", user_data.accent_color.unwrap()));
    } else {
        println!("accent_color: None");
    }
    println!("global_name: {}", user_data.global_name.unwrap_or_else(|| "None".to_string()));
    if user_data.avatar_decoration_data.is_some() {
        let avatar_decoration_data = user_data.avatar_decoration_data.clone().unwrap();
        println!("avatar_decoration_data:");
        println!(" - asset: {}", avatar_decoration_data.asset);
        println!(" - sku_id: {}", avatar_decoration_data.sku_id);
        if avatar_decoration_data.expires_at.is_some() {
            println!(" - expires_at: {}", avatar_decoration_data.expires_at.unwrap());
        } else {
            println!(" - expires_at: None");
        }
    } else {
        println!("avatar_decoration_data: None");
    }
    println!("banner_color: {}", user_data.banner_color.unwrap_or_else(|| "None".to_string()));
    if user_data.clan.is_some() {
        let clan = user_data.clan.clone().unwrap();
        println!("clan:");
        println!(" - identity_guild_id: {}", clan.identity_guild_id);
        println!(" - identity_enabled: {}", clan.identity_enabled);
        println!(" - tag: {}", clan.tag);
        println!(" - badge: {}", clan.badge);
    } else {
        println!("clan: None");
    }
    Ok(())
}

// todo cargo runしてエラーを見ろ