use scraps_libs::model::verification::{TrustTier, Verification};

/// Who confirmed a scrap's content, as its `verified` names it, with the
/// trust tier derived from every entry.
#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct VerifiedTera {
    by: String,
    tier: &'static str,
}

impl VerifiedTera {
    /// None for a scrap with no readable entry, so its page shows no line.
    pub fn new(verifications: &[Verification]) -> Option<Self> {
        let tier = TrustTier::of(verifications);
        // The actor shown is the one the tier rests on: a newer machine check
        // must not hide the human review that makes a scrap human-reviewed.
        let entry = match tier {
            TrustTier::HumanReviewed => verifications.iter().find(|v| v.is_human()),
            _ => verifications.first(),
        }?;
        Some(VerifiedTera {
            by: entry.by.clone(),
            tier: tier.as_str(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn verification(by: &str, at: &str) -> Verification {
        Verification {
            by: by.to_string(),
            at: DateTime::parse_from_rfc3339(at)
                .unwrap()
                .with_timezone(&Utc),
        }
    }

    #[test]
    fn it_shows_no_line_without_an_entry() {
        assert_eq!(VerifiedTera::new(&[]), None);
    }

    #[test]
    fn it_names_the_human_reviewer_over_a_newer_machine_check() {
        // Newest first, as the model orders them.
        let verifications = [
            verification("process:ci", "2026-10-18T00:00:00Z"),
            verification("human:alice", "2026-10-17T00:00:00Z"),
        ];
        assert_eq!(
            VerifiedTera::new(&verifications),
            Some(VerifiedTera {
                by: "human:alice".to_string(),
                tier: "human-reviewed",
            })
        );
    }

    #[test]
    fn it_names_the_newest_machine_check_when_no_human_confirmed_it() {
        let verifications = [
            verification("process:nightly", "2026-10-18T00:00:00Z"),
            verification("process:ci", "2026-10-17T00:00:00Z"),
        ];
        assert_eq!(
            VerifiedTera::new(&verifications),
            Some(VerifiedTera {
                by: "process:nightly".to_string(),
                tier: "machine-confirmed",
            })
        );
    }
}
