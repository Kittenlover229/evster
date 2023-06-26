use crate::{Action, World};

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum ActionStatus {
    // An action can be performed
    Accepted,
    // An action can not be performed due to some reason
    // TODO: provide reason
    Rejected {
        rule_name: Option<String>,
        display_reason: String,
    },
}

pub trait RulePredicate = for<'a> Fn(&'a World, &'a Action) -> ActionStatus + 'static + Send + Sync;

pub struct Rule {
    display_name: Option<String>,
    predicate: Box<dyn RulePredicate>,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            display_name: Some("Always Accepting Rule".to_string()),
            predicate: Box::new(|_: &'_ World, _: &'_ Action| ActionStatus::Accepted),
        }
    }
}

impl<F> From<F> for Rule
where
    F: RulePredicate,
{
    fn from(value: F) -> Self {
        Self {
            display_name: None,
            predicate: Box::new(value),
        }
    }
}

impl Rule {
    pub fn check(&self, world: &World, action: &Action) -> ActionStatus {
        self.predicate.as_ref()(world, action)
    }

    pub fn display_name_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.display_name
            .as_ref()
            .map(String::as_str)
            .unwrap_or(default)
    }
}
