use cntp_i18n::{I18nString, I18nStringPart};

enum StateMachine {
    Idle(String),
    OneBrace(String),
    Variable(String),
    VariableClosing(String),
}

#[derive(Clone)]
pub enum I18nFullStringPart {
    Static(I18nString),
    Variable(I18nString),
    Count,
}

impl I18nFullStringPart {
    pub fn calculate_string_part(&self, _this_key: &str) -> I18nStringPart {
        match self {
            I18nFullStringPart::Static(i18n_string) => I18nStringPart::Static(i18n_string.clone()),
            I18nFullStringPart::Variable(i18n_string) => {
                // let variables = &variable_list()[this_key];
                I18nStringPart::Variable(
                    i18n_string.clone(),
                    0, // variables
                       //     .iter()
                       //     .position(|string_to_check| **string_to_check == **i18n_string)
                       //     .expect("Expected variable list to contain all present variables"),
                )
            }
            I18nFullStringPart::Count => {
                // let variables = &variable_list()[this_key];
                I18nStringPart::Count(
                    1, // variables
                      //     .iter()
                      //     .position(|variable| variable == "count")
                      //     .expect("Expected variable list to contain a count variable"),
                )
            }
        }
    }
}

pub fn parse_raw_string(string: &str) -> Vec<I18nFullStringPart> {
    let mut parts = Vec::new();
    let mut state_machine = StateMachine::Idle(String::default());
    for char in string.chars() {
        match state_machine {
            StateMachine::Idle(string) => {
                if char == '{' {
                    state_machine = StateMachine::OneBrace(string);
                } else {
                    state_machine = StateMachine::Idle(format!("{string}{char}"));
                }
            }
            StateMachine::OneBrace(string) => {
                if char == '{' {
                    parts.push(I18nFullStringPart::Static(string.into()));
                    state_machine = StateMachine::Variable(String::default());
                } else {
                    state_machine = StateMachine::Idle(format!("{string}{{{char}"));
                }
            }
            StateMachine::Variable(string) => {
                if char == '}' {
                    state_machine = StateMachine::VariableClosing(string);
                } else {
                    state_machine = StateMachine::Variable(format!("{string}{char}"));
                }
            }
            StateMachine::VariableClosing(string) => {
                if char == '}' {
                    if string == "count" {
                        parts.push(I18nFullStringPart::Count);
                    } else {
                        parts.push(I18nFullStringPart::Variable(string.into()));
                    }
                    state_machine = StateMachine::Idle(String::default());
                } else {
                    state_machine = StateMachine::Idle(format!("{string}}}{char}"));
                }
            }
        }
    }

    match state_machine {
        StateMachine::Idle(string) => {
            if !string.is_empty() {
                parts.push(I18nFullStringPart::Static(string.into()));
            }
        }
        StateMachine::OneBrace(string) => {
            parts.push(I18nFullStringPart::Static(format!("{string}{{").into()));
        }
        StateMachine::Variable(string) => {
            parts.push(I18nFullStringPart::Static(format!("{{{{{string}").into()));
        }
        StateMachine::VariableClosing(string) => {
            parts.push(I18nFullStringPart::Static(format!("{{{{{string}}}").into()));
        }
    }

    parts
}
