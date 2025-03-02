use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};
use time::OffsetDateTime;

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DecisionsResponse {
    #[serde_as(deserialize_as = "DefaultOnNull<Vec<_>>")]
    pub new: Vec<Decision>,
    #[serde_as(deserialize_as = "DefaultOnNull<Vec<_>>")]
    pub deleted: Vec<Decision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Decision {
    /// the duration of the decisions
    pub duration: String,
    /// (only relevant for GET ops) the unique id
    pub id: Option<i64>,
    /// the origin of the decision : cscli, crowdsec
    pub origin: Origin,
    pub scenario: String,
    /// the scope of decision : does it apply to an IP, a range, a username, etc
    pub scope: Scope,
    /// true if the decision result from a scenario in simulation mode
    pub simulated: Option<bool>,
    /// the type of decision, might be 'ban', 'captcha' or something custom. Ignored when watcher (cscli/crowdsec) is pushing to APIL.
    #[serde(rename = "type")]
    pub type_: DecisionType,
    /// the date until the decisions must be active
    pub until: Option<OffsetDateTime>,
    /// only relevant for LAPI->CAPI, ignored for cscli->LAPI and crowdsec->LAPI
    pub uuid: Option<String>,
    /// the value of the decision scope : an IP, a range, a username, etc
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    #[default]
    Cscli,
    Crowdsec,
    #[serde(rename = "CAPI")]
    Capi,
    Lists,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cscli => f.write_str("cscli"),
            Self::Crowdsec => f.write_str("crowdsec"),
            Self::Capi => f.write_str("CAPI"),
            Self::Lists => f.write_str("lists"),
            Self::Other(s) => f.write_str(s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub enum Scope {
    #[default]
    Ip,
    Range,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum DecisionType {
    #[default]
    Ban,
    Captcha,
    #[serde(untagged)]
    Other(String),
}
