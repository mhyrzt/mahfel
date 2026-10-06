use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    Poll,
    Quiz,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActivityDefinition {
    pub question: String,
    pub options: Vec<String>,
    pub kind: ActivityKind,
    pub multiple: bool,
    pub anonymous: bool,
    pub correct_options: Vec<u16>,
    pub explanation: Option<String>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ActivityError {
    #[error("question must contain 1 to 500 characters")]
    Question,
    #[error("provide 2 to 10 distinct options, each containing 1 to 200 characters")]
    Options,
    #[error("quiz must define valid correct options consistent with its selection mode")]
    CorrectOptions,
    #[error("polls cannot have correct options or explanations")]
    PollAnswerKey,
    #[error("explanation must contain at most 1000 characters")]
    Explanation,
    #[error("select valid, distinct options consistent with the selection mode")]
    Selection,
}

impl ActivityDefinition {
    pub fn normalize(mut self) -> Result<Self, ActivityError> {
        self.question = self.question.trim().to_owned();
        self.options = self
            .options
            .into_iter()
            .map(|s| s.trim().to_owned())
            .collect();
        self.explanation = self
            .explanation
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
        self.validate()?;
        self.correct_options.sort_unstable();
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), ActivityError> {
        if !(1..=500).contains(&self.question.trim().chars().count()) {
            return Err(ActivityError::Question);
        }
        let unique: std::collections::BTreeSet<_> = self.options.iter().map(|s| s.trim()).collect();
        if !(2..=10).contains(&self.options.len())
            || unique.len() != self.options.len()
            || self
                .options
                .iter()
                .any(|s| !(1..=200).contains(&s.trim().chars().count()))
        {
            return Err(ActivityError::Options);
        }
        match self.kind {
            ActivityKind::Poll
                if !self.correct_options.is_empty() || self.explanation.is_some() =>
            {
                return Err(ActivityError::PollAnswerKey);
            }
            ActivityKind::Quiz => self
                .validate_selection(&self.correct_options)
                .map_err(|_| ActivityError::CorrectOptions)?,
            ActivityKind::Poll => {}
        }
        if self
            .explanation
            .as_ref()
            .is_some_and(|s| s.chars().count() > 1000)
        {
            return Err(ActivityError::Explanation);
        }
        Ok(())
    }

    pub fn validate_selection(&self, selected: &[u16]) -> Result<(), ActivityError> {
        let unique: std::collections::BTreeSet<_> = selected.iter().collect();
        if selected.is_empty()
            || selected.len() > self.options.len()
            || unique.len() != selected.len()
            || (!self.multiple && selected.len() != 1)
            || selected
                .iter()
                .any(|i| usize::from(*i) >= self.options.len())
        {
            return Err(ActivityError::Selection);
        }
        Ok(())
    }

    #[must_use]
    pub fn is_correct(&self, selected: &[u16]) -> bool {
        self.kind == ActivityKind::Quiz
            && self.validate_selection(selected).is_ok()
            && selected
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                == self.correct_options.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quiz() -> ActivityDefinition {
        ActivityDefinition {
            question: "Which are prime?".into(),
            options: vec!["2".into(), "3".into(), "4".into()],
            kind: ActivityKind::Quiz,
            multiple: true,
            anonymous: true,
            correct_options: vec![0, 1],
            explanation: None,
        }
    }
    #[test]
    fn exact_set_grading_rejects_partial_extra_and_duplicate_answers() {
        let q = quiz();
        assert!(q.is_correct(&[1, 0]));
        for selected in [&[0][..], &[0, 1, 2], &[0, 0, 1], &[]] {
            assert!(!q.is_correct(selected));
        }
    }
    #[test]
    fn validation_checks_modes_bounds_and_trimmed_duplicates() {
        let mut q = quiz();
        assert!(q.clone().normalize().is_ok());
        q.multiple = false;
        assert_eq!(q.validate(), Err(ActivityError::CorrectOptions));
        q.multiple = true;
        q.options[1] = " 2 ".into();
        assert_eq!(q.validate(), Err(ActivityError::Options));
        q = quiz();
        q.kind = ActivityKind::Poll;
        assert_eq!(q.validate(), Err(ActivityError::PollAnswerKey));
        assert!(quiz().validate_selection(&[3]).is_err());
    }
}
