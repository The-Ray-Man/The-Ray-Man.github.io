use wasm_bindgen::prelude::wasm_bindgen;

use crate::{rule, var};
use std::{collections::HashSet, fmt::Display};
use crate::Tsify;

use super::mathjax::MathJax;

/// Utility trait to implement these functions for `Vec<RuleExpr>`
pub trait RuleInfo {
    /// Returns all variables used on the left hand side of the rule
    fn all_vars_lhs(&self) -> HashSet<usize>;
    /// Returns all variables used on the right hand side of the rule
    fn all_vars_rhs(&self) -> HashSet<usize>;
}

impl RuleInfo for Vec<RuleExpr> {
    /// Returns all variables used on the left hand side of the rules
    fn all_vars_lhs(&self) -> HashSet<usize> {
        let mut res = HashSet::new();
        for rule in self {
            res.extend(&rule.all_vars_lhs());
        }
        res
    }
    /// Returns all variables used on the left hand side of the rule
    fn all_vars_rhs(&self) -> HashSet<usize> {
        let mut res = HashSet::new();
        for rule in self {
            res.extend(&rule.all_vars_rhs());
        }
        res
    }
}


/// A type expression, it forms a recursive tree structure, therefore `Box<Type>` is needed
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum TypeExpr {
    /// A function, the type expression has the form `tX -> tY`
    Function(Box<TypeExpr>, Box<TypeExpr>),
    /// A tuple, the type expression has the form `(tX, tY)`
    Tuple(Box<TypeExpr>, Box<TypeExpr>),
    /// A variable, the type expression has the form `tX`
    Var(usize),
    /// A boolean, the type expression has the form `Bool`
    Bool,
    /// An integer, the type expression has the form `Int`
    Int,
}

impl MathJax for TypeExpr {
    fn to_mathjax(&self) -> String {
        match self {
            TypeExpr::Function(left, right) => format!("({} \\to {})", left.to_mathjax(), right.to_mathjax()),
            TypeExpr::Tuple(left, right) => format!("({}, {})", left.to_mathjax(), right.to_mathjax()),
            TypeExpr::Var(x) => format!("t_{{{}}}", x),
            TypeExpr::Bool => "Bool".to_string(),
            TypeExpr::Int => "Int".to_string(),
        }
    }
} 

impl TypeExpr {
    /// Returns `Some(X)` if it could use rule with index `X` to substitute a variable
    /// If no suitable rules were found it returns None
    pub fn substitute_constraint(&mut self, rules: &Vec<RuleExpr>) -> Option<RuleExpr> {
        match *self {
            TypeExpr::Function(ref mut left, ref mut right) => {
                if let Some(rule) = left.substitute_constraint(rules) {
                    dbg!("left");
                    dbg!(&left);
                    Some(rule)
                } else {
                    dbg!("right");
                    right.substitute_constraint(rules)
                }
            }
            TypeExpr::Tuple(ref mut left, ref mut right) => {
                if let Some(rule) = left.substitute_constraint(rules) {
                    Some(rule)
                } else {
                    right.substitute_constraint(rules)
                }
            }
            TypeExpr::Var(x) => {
                dbg!("var");
                if let Some(rule) = rules.iter().find(|r| r.var == x) {
                    let new_expr = *rule.rhs.clone();
                    *self = new_expr;
                    Some((*rule).clone())
                } else {
                    None
                }
            }
            TypeExpr::Bool => None,
            TypeExpr::Int => None,
        }
    }
    /// Returns if the type expression needs to be wrapped in parenthesis
    pub fn needs_wrapping(&self) -> bool {
        match &self {
            TypeExpr::Function(_, _) => true,
            TypeExpr::Tuple(_, _) => false,
            TypeExpr::Var(_) => false,
            TypeExpr::Bool => false,
            TypeExpr::Int => false,
        }
    }
    /// Replaces a variable recursively in the type expression
    pub fn replace_var(&mut self, from: usize, to: usize) {
        match self {
            TypeExpr::Function(l, r) => {
                l.replace_var(from, to);
                r.replace_var(from, to);
            }
            TypeExpr::Tuple(l, r) => {
                l.replace_var(from, to);
                r.replace_var(from, to);
            }
            TypeExpr::Var(x) => {
                if *x == from {
                    *self = TypeExpr::Var(to);
                }
            }
            TypeExpr::Bool => (),
            TypeExpr::Int => (),
        }
    }
    /// Return all vars contained in the right side
    pub fn all_vars(&self) -> HashSet<usize> {
        let mut res = HashSet::new();
        match self {
            TypeExpr::Function(left, right) => {
                res.extend(&left.all_vars());
                res.extend(&right.all_vars());
            }
            TypeExpr::Tuple(left, right) => {
                res.extend(&left.all_vars());
                res.extend(&right.all_vars());
            }
            TypeExpr::Var(x) => {
                res.insert(*x);
            }
            TypeExpr::Int => (),
            TypeExpr::Bool => (),
        }
        res
    }
    /// Compares types and returns all new constraints generated by the comparison
    pub fn compare_types(&self, other: &TypeExpr) -> Result<Vec<RuleExpr>, ()> {
        match &self {
            TypeExpr::Function(sleft, sright) => match other {
                TypeExpr::Function(oleft, oright) => {
                    let mut rules = sleft.compare_types(oleft)?;
                    rules.append(&mut sright.compare_types(oright)?);
                    Ok(rules)
                }
                TypeExpr::Var(x) => Ok(vec![rule!(*x, Box::new(self.clone()))]),
                _ => Err(()),
            },
            TypeExpr::Tuple(sleft, sright) => match other {
                TypeExpr::Tuple(oleft, oright) => {
                    let mut rules = sleft.compare_types(oleft)?;
                    rules.append(&mut sright.compare_types(oright)?);
                    Ok(rules)
                }
                TypeExpr::Var(x) => Ok(vec![rule!(*x, Box::new(self.clone()))]),
                _ => Err(()),
            },
            TypeExpr::Var(x) => match other {
                TypeExpr::Var(c) => {
                    // we only want rules where left variable ID is smaller than variable ID
                    // they will be replaced afterwards
                    if x < c {
                        Ok(vec![rule!(*x, var!(*c))])
                    } else if c < x {
                        Ok(vec![rule!(*c, var!(*x))])
                    } else {
                        // c == x and no rule needs to be inserted
                        Ok(vec![])
                    }
                }
                _ => Ok(vec![rule!(*x, Box::new(other.clone()))]),
            },
            TypeExpr::Bool => match other {
                TypeExpr::Var(x) => Ok(vec![rule!(*x, Box::new(self.clone()))]),
                TypeExpr::Bool => Ok(vec![]),
                _ => Err(()),
            },
            TypeExpr::Int => match other {
                TypeExpr::Var(x) => Ok(vec![rule!(*x, Box::new(self.clone()))]),
                TypeExpr::Int => Ok(vec![]),
                _ => Err(()),
            },
        }
    }
}

impl Display for TypeExpr {
    /// Recursively display the type expression, wraps some variants in parenthesis
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TypeExpr::Function(t1, t2) => {
                let t1_out = if t1.needs_wrapping() {
                    format!("({t1})")
                } else {
                    format!("{t1}")
                };
                // since -> is right associative, no need to put parenthesis around the right part
                write!(f, "{t1_out} -> {t2}")
            }
            TypeExpr::Tuple(t1, t2) => {
                let t1_out = if t1.needs_wrapping() {
                    format!("({t1})")
                } else {
                    format!("{t1}")
                };
                let t2_out = if t2.needs_wrapping() {
                    format!("({t2})")
                } else {
                    format!("{t2}")
                };
                write!(f, "({t1_out}, {t2_out})")
            }
            TypeExpr::Var(x) => write!(f, "t{x}"),
            TypeExpr::Bool => write!(f, "Bool"),
            TypeExpr::Int => write!(f, "Int"),
        }
    }
}

/// A single rule expression, with a left hand side variable and a type expression on the right hand side
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct RuleExpr {
    /// The variable ID of the left hand side
    pub var: usize,
    /// The type expression on the right hand side
    pub rhs: Box<TypeExpr>,
}

impl MathJax for RuleExpr {
    fn to_mathjax(&self) -> String {
        format!("t_{{{}}} = {}", self.var, self.rhs.to_mathjax())
    }
}

impl RuleExpr {
    /// Checks if a rule is of the form `tX = tY` for some `X, Y`.
    /// If the rule is simple, it returns `Some(X, Y)`
    pub fn is_simple(&self) -> Option<(usize, usize)> {
        if let TypeExpr::Var(r) = *self.rhs {
            Some((self.var, r))
        } else {
            None
        }
    }
    /// Recursively replaces all variables with ID `from` with ID `to`
    /// Handles both left hand side and right hand side
    pub fn replace_var(&mut self, from: usize, to: usize) {
        if self.var == from {
            self.var = to;
        }
        self.rhs.replace_var(from, to);
    }
    /// Substitutes the first variable with a known constraint, mutates in place and only substitutes the first occurence
    pub fn substitute_constraint(&mut self, rules: &Vec<RuleExpr>) -> Option<RuleExpr> {
        let res = self.rhs.substitute_constraint(rules);
        dbg!(&self);
        res
    }
    /// Compares rules and returns all new constraints generated by the comparison
    pub fn compare_rules(&self, other: &RuleExpr) -> Result<Vec<RuleExpr>, ()> {
        self.rhs.compare_types(&*other.rhs)
    }
    /// Checks if both rules have the same left hand side variable
    pub fn has_same_lhs(&self, other: &RuleExpr) -> bool {
        self.var == other.var
    }
    /// Checks if rule is a constraint for the given left hand side
    pub fn has_lhs(&self, other: usize) -> bool {
        self.var == other
    }
}

impl RuleInfo for RuleExpr {
    /// Return all variables on the left hand side of the rule, which is only one
    fn all_vars_lhs(&self) -> HashSet<usize> {
        HashSet::from([self.var])
    }
    /// Recursively go through all variables in the right hand side and collect them
    fn all_vars_rhs(&self) -> HashSet<usize> {
        self.rhs.all_vars()
    }
}

impl Display for RuleExpr {
    /// Assumes that variables are being displayed in the form `t0` for variable ID `0`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t{} = {}", self.var, self.rhs)
    }
}