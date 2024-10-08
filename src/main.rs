use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt, join};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    token: String,
}

impl Config {
    async fn get() -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open("config.json").await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let config: Self = serde_json::from_str(&contents)?;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    AvatarDecoration,
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImageType::Avatar => write!(f, "avatars"),
            ImageType::Banner => write!(f, "banners"),
            ImageType::AvatarDecoration => write!(f, "avatar-decoration-presets"),
        }
    }
}

impl UserData {
    async fn get(token: &str, user_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();
        let mut header = HeaderMap::new();
        header.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_value = format!("Bot {}", token);
        header.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);
        let res = client.get(&format!("https://discord.com/api/v10/users/{}", user_id))
            .headers(header)
            .send().await?;
        let body = res.text().await?;
        let user_data: Self = serde_json::from_str(&body)?;
        Ok(user_data)
    }
    async fn id_to_link(&self, img_type: ImageType) -> Result<String, Box<dyn std::error::Error>> {
        let img_id;
        if img_type == ImageType::Avatar && self.avatar.is_none() {
            return Ok("https://cdn.discordapp.com/embed/avatars/0.png".to_string());
        } else if img_type == ImageType::Banner && self.banner.is_none() {
            return Ok("None".to_string());
        } else if img_type == ImageType::AvatarDecoration && self.avatar_decoration_data.is_none() {
            return Ok("None".to_string());
        } else {
            img_id = match img_type {
                ImageType::Avatar => self.avatar.clone().unwrap(),
                ImageType::Banner => self.banner.clone().unwrap(),
                ImageType::AvatarDecoration => self.avatar_decoration_data.clone().unwrap().asset,
            };
        }
        let mut url = String::new();
        if img_type == ImageType::Avatar || img_type == ImageType::Banner {
            url = format!("https://cdn.discordapp.com/{}/{}/{}", &img_type, self.id, img_id)
        } else if img_type == ImageType::AvatarDecoration {
            return Ok(format!("https://cdn.discordapp.com/{}/{}.png?size=4096", &img_type, img_id));
        }

        url.push_str(".gif");
        let response = reqwest::get(&url).await?;
        return if response.status().is_success() {
            url.push_str("?size=4096");
            Ok(url)
        } else {
            url.truncate(url.len() - 4);
            url.push_str(".png?size=4096");
            Ok(url)
        };
    }
    fn check_flags(&self) -> Vec<String> {
        const FLAGS: &[(&str, u64)] = &[
            ("Staff", 1),
            ("Partnered_Server_Owner", 2),
            ("HypeSquad_Events", 4),
            ("Bug_Hunter_Level_1", 8),
            ("HypeSquad_Bravery", 64),
            ("HypeSquad_Brilliance", 128),
            ("HypeSquad_Balance", 256),
            ("Premium_Early_Supporter", 512),
            ("Team_Pseudo_User", 1024),
            ("Bug_Hunter_Level_2", 16384),
            ("Verified_Bot", 65536),
            ("Verified_Developer", 131072),
            ("Certified_Moderator", 262144),
            ("Bot_Http_Interactions", 524288),
            ("Active_Developer", 4194304)
        ];

        FLAGS.iter()
            .filter_map(|&(flag_name, flag_value)| {
                if &self.public_flags & flag_value == flag_value {
                    Some(flag_name.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = DefaultEditor::new()?;
    let config = Config::get().await?;
    loop {
        let id = rl.readline("ID: ")?;
        println!();

        let user_data = UserData::get(&config.token, &id).await?;


        let (avatar, banner, asset) = join!(
            user_data.id_to_link(ImageType::Avatar),
            user_data.id_to_link(ImageType::Banner),
            user_data.id_to_link(ImageType::AvatarDecoration)
        );

        println!("ID: {}", user_data.id);
        println!("Username: {}", user_data.username);
        println!("Avatar: {}", avatar?);
        println!("Discriminator: {}", user_data.discriminator);
        println!("Public Flags: {}", user_data.public_flags);
        if user_data.public_flags != 0 {
            println!("Badge:");
            for flag in user_data.check_flags() {
                println!(" - {}", flag);
            }
        } else {
            println!("Badge: None");
        }
        println!("Flags: {}", user_data.flags);
        println!("Bot: {}", user_data.bot.unwrap_or_else(|| false));
        println!("Banner: {}", banner?);
        if user_data.accent_color.is_some() {
            println!("Accent Color: {}", format!("#{:06x}", user_data.accent_color.unwrap()));
        } else {
            println!("Accent Color: None");
        }
        println!("Global Name: {}", user_data.global_name.unwrap_or_else(|| "None".to_string()));
        if user_data.avatar_decoration_data.is_some() {
            let avatar_decoration_data = user_data.avatar_decoration_data.clone().unwrap();
            println!("Avatar Decoration Data:");
            println!(" - Asset: {}", asset?);
            println!(" - SKU ID: {}", avatar_decoration_data.sku_id);
            if avatar_decoration_data.expires_at.is_some() {
                println!(" - Expires at: {}", avatar_decoration_data.expires_at.unwrap());
            } else {
                println!(" - Expires at: None");
            }
        } else {
            println!("Avatar Decoration Data: None");
        }
        println!("Banner Color: {}", user_data.banner_color.unwrap_or_else(|| "None".to_string()));
        if user_data.clan.is_some() {
            let clan = user_data.clan.clone().unwrap();
            println!("Clan:");
            println!(" - Identity Guild Id: {}", clan.identity_guild_id);
            println!(" - Identity Enabled: {}", clan.identity_enabled);
            println!(" - Tag: {}", clan.tag);
            println!(" - Badge: {}", clan.badge);
        } else {
            println!("Clan: None");
        }
        println!();
    }
}
