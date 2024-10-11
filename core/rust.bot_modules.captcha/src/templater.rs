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

    pub fn create_captcha(&self) -> Result<(String, Vec<u8>), silverpelt::Error> {
        let mut c = captcha::Captcha::new();
        c.add_random_chars(self.char_count as u32);

        if let Some(set_viewbox_at_idx) = self.set_viewbox_at_idx {
            // Do two separate for loops, one for 0..set_viewbox_at_idx and one for set_viewbox_at_idx..filters.len()
            for f in self.filters.iter().take(set_viewbox_at_idx) {
                c.apply_filter_dyn(f)?;
            }

            c.view(self.viewbox_size.0, self.viewbox_size.1);

            for f in self.filters.iter().skip(set_viewbox_at_idx) {
                c.apply_filter_dyn(f)?;
            }
        } else {
            for f in self.filters.iter() {
                c.apply_filter_dyn(f)?;
            }
        }

        Ok(c.as_tuple().ok_or("Failed to create captcha")?)
    }
}
