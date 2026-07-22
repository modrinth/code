#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FilterExpr {
    And(Vec<Self>),
    Or(Vec<Self>),
    Predicate(FilterPredicate),
}

impl FilterExpr {
    pub fn and(expressions: impl IntoIterator<Item = Self>) -> Option<Self> {
        let mut expressions = expressions.into_iter().collect::<Vec<_>>();
        match expressions.len() {
            0 => None,
            1 => expressions.pop(),
            _ => Some(Self::And(expressions)),
        }
    }

    pub fn or(expressions: impl IntoIterator<Item = Self>) -> Option<Self> {
        let mut expressions = expressions.into_iter().collect::<Vec<_>>();
        match expressions.len() {
            0 => None,
            1 => expressions.pop(),
            _ => Some(Self::Or(expressions)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FilterPredicate {
    pub field: FilterField,
    pub condition: FilterCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FilterField(String);

impl FilterField {
    pub fn new(field: impl Into<String>) -> Self {
        Self(field.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FilterCondition {
    Compare {
        comparison: FilterComparison,
        value: FilterLiteral,
    },
    In {
        values: Vec<FilterLiteral>,
        negated: bool,
    },
    Exists {
        negated: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FilterComparison {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FilterLiteral {
    String(String),
    Number(String),
    Bool(bool),
}

impl FilterLiteral {
    pub(super) fn from_bare(value: String) -> Self {
        if value.eq_ignore_ascii_case("true") {
            Self::Bool(true)
        } else if value.eq_ignore_ascii_case("false") {
            Self::Bool(false)
        } else if value.parse::<f64>().is_ok() {
            Self::Number(value)
        } else {
            Self::String(value)
        }
    }
}
