#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflectionTrigger {
    Conflict,
    Failure,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflectionDecision {
    MarkDisputed,
    SupersedeWithReplacement,
    RecordOnly,
}

pub fn classify_reflection(trigger: ReflectionTrigger) -> ReflectionDecision {
    match trigger {
        ReflectionTrigger::Conflict => ReflectionDecision::MarkDisputed,
        ReflectionTrigger::Failure => ReflectionDecision::SupersedeWithReplacement,
        ReflectionTrigger::Manual => ReflectionDecision::RecordOnly,
    }
}
