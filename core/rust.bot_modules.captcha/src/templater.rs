use captcha::filters::Filter;

pub const MAX_FILTERS: usize = 12;
pub const MAX_VIEWBOX_X: u32 = 512;
pub const MAX_VIEWBOX_Y: u32 = 512;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CaptchaConfig {
    pub char_count: u8,
    pub filters: Vec<Box<dyn Filter>>,
    pub viewbox_size: (u32, u32),
    pub set_viewbox_at_idx: Option<usize>,
}

impl CaptchaConfig {
    pub fn is_valid(&self) -> bool {
        self.char_count > 0 
        && self.filters.len() <= MAX_FILTERS // Max filters check
        && self.viewbox_size.0 > 0 && self.viewbox_size.0 < MAX_VIEWBOX_X // Max viewbox size check (X)
        && self.viewbox_size.1 > 0 && self.viewbox_size.1 < MAX_VIEWBOX_Y // Max viewbox size check (Y)
        && self.set_viewbox_at_idx.map_or(true, |idx| idx < self.filters.len()) // Check set_viewbox_at_idx
        && self.filters.iter().all(|f| f.validate(self.viewbox_size).is_ok()) // Check filters
    }

    pub async fn create_captcha(
        self,
        timeout: std::time::Duration,
    ) -> Result<(String, Vec<u8>), silverpelt::Error> {
        if !self.is_valid() {
            return Err("Invalid captcha configuration".into());
        }

        let start_time = std::time::Instant::now();
        
        tokio::task::spawn_blocking(move || {
            let mut c = captcha::Captcha::new();
            c.add_random_chars(self.char_count as u32);
    
            if let Some(set_viewbox_at_idx) = self.set_viewbox_at_idx {
                // Do two separate for loops, one for 0..set_viewbox_at_idx and one for set_viewbox_at_idx..filters.len()
                for f in self.filters.iter().take(set_viewbox_at_idx) {
                    // Check if we've exceeded the timeout
                    if start_time - std::time::Instant::now() > timeout {
                        return Err(format!("Timeout exceeded when rendering captcha: {:?}", timeout).into());
                    }

                    c.apply_filter_dyn(f)?;
                }
    
                c.view(self.viewbox_size.0, self.viewbox_size.1);

                // Check if we've exceeded the timeout
                if start_time - std::time::Instant::now() > timeout {
                    return Err(format!("Timeout exceeded when rendering captcha: {:?}", timeout).into());
                }

                for f in self.filters.iter().skip(set_viewbox_at_idx) {
                    // Check if we've exceeded the timeout
                    if start_time - std::time::Instant::now() > timeout {
                        return Err(format!("Timeout exceeded when rendering captcha: {:?}", timeout).into());
                    }

                    c.apply_filter_dyn(f)?;
                }
            } else {
                c.view(self.viewbox_size.0, self.viewbox_size.1);

                for f in self.filters.iter() {
                    // Check if we've exceeded the timeout
                    if start_time - std::time::Instant::now() > timeout {
                        return Err(format!("Timeout exceeded when rendering captcha: {:?}", timeout).into());
                    }

                    c.apply_filter_dyn(f)?;
                }
            }
    
            Ok(c.as_tuple().ok_or("Failed to create captcha")?)    
        })
        .await?
    }
}

/// A CaptchaContext is a context for captcha's
/// that can be accessed in captcha templates
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptchaContext {
    /// The user ID that triggered the captcha
    pub user_id: serenity::all::UserId,
    /// The guild ID that the user triggered the captcha in
    pub guild_id: serenity::all::GuildId,
    /// The channel ID that the user triggered the captcha in. May be None in some cases (captcha not in channel)
    pub channel_id: Option<serenity::all::ChannelId>,
}

#[typetag::serde]
impl templating::Context for CaptchaContext {}