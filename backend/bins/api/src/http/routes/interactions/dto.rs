use mahfel_application::{
    interactions::results,
    ports::{ActivityRecord, ActivityStatus, InteractionSnapshot},
};
use mahfel_domain::{ActivityDefinition, ActivityKind, ActorId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityKindDto {
    Poll,
    Quiz,
}
impl From<ActivityKind> for ActivityKindDto {
    fn from(kind: ActivityKind) -> Self {
        match kind {
            ActivityKind::Poll => Self::Poll,
            ActivityKind::Quiz => Self::Quiz,
        }
    }
}
#[derive(Deserialize, ToSchema)]
pub(crate) struct ActivityDefinitionRequest {
    pub question: String,
    pub options: Vec<String>,
    pub kind: ActivityKindDto,
    pub multiple: bool,
    pub anonymous: bool,
    pub correct_options: Vec<u16>,
    pub explanation: Option<String>,
}
impl From<ActivityDefinitionRequest> for ActivityDefinition {
    fn from(d: ActivityDefinitionRequest) -> Self {
        Self {
            question: d.question,
            options: d.options,
            kind: match d.kind {
                ActivityKindDto::Poll => ActivityKind::Poll,
                ActivityKindDto::Quiz => ActivityKind::Quiz,
            },
            multiple: d.multiple,
            anonymous: d.anonymous,
            correct_options: d.correct_options,
            explanation: d.explanation,
        }
    }
}
#[derive(Deserialize, ToSchema)]
pub(crate) struct InteractionRequest {
    pub request_id: Uuid,
    #[serde(flatten)]
    pub command: InteractionAction,
}
#[derive(Deserialize, ToSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub(crate) enum InteractionAction {
    SaveDraft {
        id: Uuid,
        definition: ActivityDefinitionRequest,
    },
    Launch {
        id: Uuid,
    },
    Close {
        id: Uuid,
    },
    Vote {
        id: Uuid,
        selected: Vec<u16>,
        expected_version: i64,
    },
    Retract {
        id: Uuid,
        expected_version: i64,
    },
    SetHand {
        raised: bool,
        expected_version: i64,
    },
    LowerHand {
        participant_id: Uuid,
        expected_order: i64,
    },
    ClearHands {
        expected_version: i64,
    },
}
impl From<InteractionAction> for mahfel_application::ports::InteractionCommand {
    fn from(action: InteractionAction) -> Self {
        match action {
            InteractionAction::SaveDraft { id, definition } => Self::SaveDraft {
                id,
                definition: definition.into(),
            },
            InteractionAction::Launch { id } => Self::Launch { id },
            InteractionAction::Close { id } => Self::Close { id },
            InteractionAction::Vote {
                id,
                selected,
                expected_version,
            } => Self::Vote {
                id,
                selected,
                expected_version,
            },
            InteractionAction::Retract {
                id,
                expected_version,
            } => Self::Retract {
                id,
                expected_version,
            },
            InteractionAction::SetHand {
                raised,
                expected_version,
            } => Self::SetHand {
                raised,
                expected_version,
            },
            InteractionAction::LowerHand {
                participant_id,
                expected_order,
            } => Self::LowerHand {
                participant_id: mahfel_domain::ParticipantId::from_uuid(participant_id),
                expected_order,
            },
            InteractionAction::ClearHands { expected_version } => {
                Self::ClearHands { expected_version }
            }
        }
    }
}
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityStatusDto {
    Draft,
    Open,
    Closed,
}
#[derive(Serialize, ToSchema)]
pub(crate) struct ActivityResultsResponse {
    respondents: usize,
    counts: Vec<usize>,
    correct_options: Option<Vec<u16>>,
    explanation: Option<String>,
    correct: Option<bool>,
}
#[derive(Serialize, ToSchema)]
pub(crate) struct ActivityResponse {
    id: Uuid,
    question: String,
    options: Vec<String>,
    kind: ActivityKindDto,
    multiple: bool,
    anonymous: bool,
    status: ActivityStatusDto,
    selected: Vec<u16>,
    vote_version: i64,
    submitted: bool,
    participation_count: Option<usize>,
    draft_correct_options: Option<Vec<u16>>,
    draft_explanation: Option<String>,
    results: Option<ActivityResultsResponse>,
}
impl ActivityResponse {
    fn new(activity: ActivityRecord, actor: ActorId, manager: bool) -> Self {
        let own = activity.votes.iter().find(|v| v.actor_id == actor);
        let selected = own.map_or_else(Vec::new, |v| v.selected.clone());
        let vote_version = own.map_or(0, |v| v.version);
        let results = results(&activity, actor).map(|r| ActivityResultsResponse {
            respondents: r.respondents,
            counts: r.counts,
            correct_options: r.correct_options,
            explanation: r.explanation,
            correct: r.correct,
        });
        let draft = activity.status == ActivityStatus::Draft && manager;
        Self {
            id: activity.id,
            question: activity.definition.question,
            options: activity.definition.options,
            kind: activity.definition.kind.into(),
            multiple: activity.definition.multiple,
            anonymous: activity.definition.anonymous,
            status: match activity.status {
                ActivityStatus::Draft => ActivityStatusDto::Draft,
                ActivityStatus::Open => ActivityStatusDto::Open,
                ActivityStatus::Closed => ActivityStatusDto::Closed,
            },
            submitted: !selected.is_empty(),
            selected,
            vote_version,
            participation_count: manager.then(|| {
                activity
                    .votes
                    .iter()
                    .filter(|v| !v.selected.is_empty())
                    .count()
            }),
            draft_correct_options: draft.then_some(activity.definition.correct_options),
            draft_explanation: if draft {
                activity.definition.explanation
            } else {
                None
            },
            results,
        }
    }
}
#[derive(Serialize, ToSchema)]
pub(crate) struct HandResponse {
    pub participant_id: String,
    pub actor_id: String,
    pub name: String,
    pub order: i64,
}
#[derive(Serialize, ToSchema)]
pub(crate) struct InteractionsResponse {
    pub version: i64,
    pub live: bool,
    pub can_manage: bool,
    pub can_moderate: bool,
    pub can_respond: bool,
    pub activities: Vec<ActivityResponse>,
    pub hands: Vec<HandResponse>,
}
impl InteractionsResponse {
    pub fn new(s: InteractionSnapshot, actor: ActorId) -> Self {
        Self {
            version: s.version,
            live: s.live,
            can_manage: s.can_manage,
            can_moderate: s.can_moderate,
            can_respond: s.can_respond,
            activities: s
                .activities
                .into_iter()
                .map(|a| ActivityResponse::new(a, actor, s.can_manage))
                .collect(),
            hands: s
                .hands
                .into_iter()
                .map(|h| HandResponse {
                    participant_id: h.participant_id.to_string(),
                    actor_id: h.actor_id.to_string(),
                    name: h.name,
                    order: h.order,
                })
                .collect(),
        }
    }
}
#[derive(Serialize, ToSchema)]
pub(crate) struct VoterResponse {
    pub name: String,
    pub actor_id: String,
    pub selected: Vec<u16>,
}
#[derive(Serialize, ToSchema)]
pub(crate) struct VotersResponse {
    pub voters: Vec<VoterResponse>,
    pub next_offset: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mahfel_application::ports::ActivityVote;
    #[test]
    fn open_quiz_response_withholds_answers_and_distribution_from_manager() {
        let actor = ActorId::new();
        let a = ActivityRecord {
            id: Uuid::new_v4(),
            status: ActivityStatus::Open,
            definition: ActivityDefinition {
                question: "Pick".into(),
                options: vec!["A".into(), "B".into()],
                kind: ActivityKind::Quiz,
                multiple: true,
                anonymous: true,
                correct_options: vec![0, 1],
                explanation: Some("Secret explanation".into()),
            },
            votes: vec![ActivityVote {
                actor_id: actor,
                name: "Private voter".into(),
                selected: vec![0],
                version: 1,
            }],
        };
        let response =
            serde_json::to_value(ActivityResponse::new(a, actor, true)).expect("serialize");
        assert!(response["results"].is_null());
        assert!(response["draft_correct_options"].is_null());
        assert!(response["draft_explanation"].is_null());
        assert!(!response.to_string().contains("Private voter"));
        assert_eq!(response["selected"], serde_json::json!([0]));
    }
}
