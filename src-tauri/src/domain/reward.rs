//! Quest reward — replaces the `"700 Orbs"` string magic.

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reward {
    Orbs(u32),
    Other(String),
}

impl Reward {
    /// The display form the UI already expects (e.g. `"700 Orbs"`).
    pub fn to_display(&self) -> String {
        match self {
            Reward::Orbs(n) => format!("{n} Orbs"),
            Reward::Other(label) => label.clone(),
        }
    }
}

impl FromStr for Reward {
    type Err = ();

    /// Parse a display string back into a typed reward. `"700 Orbs"` →
    /// `Orbs(700)`; anything else falls back to `Other`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if let Some(num) = s
            .strip_suffix(" Orbs")
            .or_else(|| s.strip_suffix(" orbs"))
        {
            if let Ok(n) = num.trim().parse::<u32>() {
                return Ok(Reward::Orbs(n));
            }
        }
        Ok(Reward::Other(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbs_display() {
        assert_eq!(Reward::Orbs(700).to_display(), "700 Orbs");
        assert_eq!(Reward::Other("1 Gem".into()).to_display(), "1 Gem");
    }

    #[test]
    fn parse_orbs_round_trip() {
        assert_eq!("700 Orbs".parse::<Reward>().unwrap(), Reward::Orbs(700));
        assert_eq!(Reward::Orbs(700).to_display().parse::<Reward>().unwrap(), Reward::Orbs(700));
    }

    #[test]
    fn parse_other_fallback() {
        assert_eq!(
            "1 Gem".parse::<Reward>().unwrap(),
            Reward::Other("1 Gem".into())
        );
    }
}
