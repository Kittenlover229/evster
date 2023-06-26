use log::trace;

use crate::{Action, ActionStatus, Grid, Rule};

pub struct World {
    pub grid: Grid,
    rules: Vec<Rule>,
}

impl World {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: Grid::new(width, height),
            rules: vec![],
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn commit_action(&mut self, action: Action) -> ActionStatus {
        use ActionStatus::*;

        match self.check_action(&action) {
            ActionStatus::Accepted => todo!(),
            ActionStatus::Rejected {
                rule_name,
                display_reason,
            } => {
                let name = rule_name
                    .as_ref()
                    .map(String::to_string)
                    .unwrap_or_else(|| "*no name*".to_string());

                trace!("Action {action:?} rejected by rule {name} (reason: {display_reason})",);

                return Rejected {
                    rule_name,
                    display_reason,
                };
            }
        }
    }

    pub fn check_action(&mut self, action: &Action) -> ActionStatus {
        use ActionStatus::*;
        for rule in &self.rules {
            match rule.check(self, action) {
                Rejected {
                    display_reason,
                    rule_name,
                } => {
                    return ActionStatus::Rejected {
                        rule_name,
                        display_reason,
                    }
                }
                Accepted => continue,
            }
        }

        Accepted
    }
}

pub fn default_rules() -> Vec<Rule> {
    vec![Rule::default()]
}
