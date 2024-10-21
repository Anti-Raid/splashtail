use captcha::filters::Filter;

pub const MAX_CHAR_COUNT: u8 = 10;
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
    pub fn is_valid(&self) -> Result<(), silverpelt::Error> {
        if self.char_count == 0 {
            return Err("char_count must be greater than 0".into());
        }

        if self.char_count > MAX_CHAR_COUNT {
            return Err(format!(
                "char_count must be less than or equal to {}",
                MAX_CHAR_COUNT
            )
            .into());
        }

        if self.filters.len() > MAX_FILTERS {
            return Err(format!("filters must be less than or equal to {}", MAX_FILTERS).into());
        }

        if self.viewbox_size.0 == 0 || self.viewbox_size.0 >= MAX_VIEWBOX_X {
            return Err(format!(
                "viewbox_size.0 must be greater than 0 and less than {}",
                MAX_VIEWBOX_X
            )
            .into());
        }

        if self.viewbox_size.1 == 0 || self.viewbox_size.1 >= MAX_VIEWBOX_Y {
            return Err(format!(
                "viewbox_size.1 must be greater than 0 and less than {}",
                MAX_VIEWBOX_Y
            )
            .into());
        }

        if let Some(set_viewbox_at_idx) = self.set_viewbox_at_idx {
            if set_viewbox_at_idx >= self.filters.len() {
                return Err("set_viewbox_at_idx must be less than the length of filters".into());
            }
        }

        for f in self.filters.iter() {
            f.validate(self.viewbox_size)?;
        }

        Ok(())
    }

    pub async fn create_captcha(
        self,
        timeout: std::time::Duration,
    ) -> Result<(String, Vec<u8>), silverpelt::Error> {
        self.is_valid()?;

        let start_time = std::time::Instant::now();

        tokio::task::spawn_blocking(move || {
            let mut c = captcha::Captcha::new();
            c.add_random_chars(self.char_count as u32);

            if let Some(set_viewbox_at_idx) = self.set_viewbox_at_idx {
                // Do two separate for loops, one for 0..set_viewbox_at_idx and one for set_viewbox_at_idx..filters.len()
                for f in self.filters.iter().take(set_viewbox_at_idx) {
                    // Check if we've exceeded the timeout
                    if start_time - std::time::Instant::now() > timeout {
                        return Err(format!(
                            "Timeout exceeded when rendering captcha: {:?}",
                            timeout
                        )
                        .into());
                    }

                    c.apply_filter_dyn(f)?;
                }

                c.view(self.viewbox_size.0, self.viewbox_size.1);

                // Check if we've exceeded the timeout
                if start_time - std::time::Instant::now() > timeout {
                    return Err(
                        format!("Timeout exceeded when rendering captcha: {:?}", timeout).into(),
                    );
                }

                for f in self.filters.iter().skip(set_viewbox_at_idx) {
                    // Check if we've exceeded the timeout
                    if start_time - std::time::Instant::now() > timeout {
                        return Err(format!(
                            "Timeout exceeded when rendering captcha: {:?}",
                            timeout
                        )
                        .into());
                    }

                    c.apply_filter_dyn(f)?;
                }
            } else {
                c.view(self.viewbox_size.0, self.viewbox_size.1);

                for f in self.filters.iter() {
                    // Check if we've exceeded the timeout
                    if start_time - std::time::Instant::now() > timeout {
                        return Err(format!(
                            "Timeout exceeded when rendering captcha: {:?}",
                            timeout
                        )
                        .into());
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
    /// The user that triggered the captcha
    pub user: serenity::all::User,
    /// The guild ID that the user triggered the captcha in
    pub guild_id: serenity::all::GuildId,
    /// The channel ID that the user triggered the captcha in. May be None in some cases (captcha not in channel)
    pub channel_id: Option<serenity::all::ChannelId>,
}

#[typetag::serde]
impl templating::Context for CaptchaContext {}
