use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter, Result as FmtResult};

use super::condition::{In, Is};
use super::domain::Property;
use super::error::{Error, Result};
use super::value::Value;

#[derive(Debug)]
pub struct Context<Pid: Property> {
    requested: HashSet<Pid>,
    provided: HashMap<Pid, Value>,
}

impl<Pid: Property> Context<Pid> {
    pub fn empty() -> Self {
        Context {
            requested: HashSet::new(),
            provided: HashMap::new(),
        }
    }

    pub fn request<I>(props: I) -> Self
    where
        I: IntoIterator<Item = Pid>,
    {
        return Context {
            requested: props.into_iter().collect(),
            provided: HashMap::new(),
        };
    }

    pub fn provide(&mut self, property: Pid, value: Value) -> Result<()> {
        if property.datatype() != value.datatype() {
            return Err(Error::TypeMismatch(
                property.name(),
                property.datatype(),
                value.datatype(),
            ));
        }

        if let Some(requested) = self.requested.get(&property) {
            self.provided.insert(*requested, value);
        }

        Ok(())
    }

    pub fn requested(&self) -> impl Iterator<Item = &Pid> {
        self.requested.iter()
    }

    pub fn provided(&self) -> impl Iterator<Item = &Pid> {
        self.provided.keys()
    }

    pub fn value(&self, property: Pid) -> Option<&Value> {
        self.provided.get(&property)
    }
}

impl<Pid: Property> Display for Context<Pid> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "requested: {:?}, provided: {:?}",
            self.requested, self.provided
        )
    }
}

pub type OpRef = usize;

#[derive(Debug, Clone)]
pub enum Operation<Pid: Property> {
    Const(bool),
    Is(Is<Pid>),
    In(In<Pid>),
    Not(OpRef),
    Or(OpRef, OpRef),
    And(OpRef, OpRef),
}

type RefCount = usize;
type Operations<Pid> = Vec<(Operation<Pid>, RefCount)>;

#[derive(Debug)]
pub enum Evaluated<Pid: Property> {
    Fully(bool, Operations<Pid>),
    Partially(Expression<Pid>),
}

impl<Pid: Property> Display for Evaluated<Pid> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match &self {
            &Evaluated::Fully(ref res, ref ops) => {
                write!(f, "Fully evaluated to '{0}', log: [", res)?;
                for (idx, op) in ops.iter().enumerate() {
                    if let (Operation::Const(res), _) = op {
                        write!(f, "{0}: {1}, ", idx, res)?;
                    } else {
                        write!(f, "<unevaluated>")?;
                    }
                }
                write!(f, "]")
            }
            &Evaluated::Partially(ref expr) => {
                write!(f, "Partially evaluated, expresion: {}", expr)
            }
        }
    }
}

#[derive(Debug)]
pub struct Expression<Pid: Property> {
    ops: Operations<Pid>,
}

impl<Pid: Property> Expression<Pid> {
    fn display(&self, root: Option<OpRef>) -> String {
        let last = if self.ops.is_empty() {
            0
        } else {
            self.ops.len() - 1
        };

        let rootref = root.unwrap_or(last);

        if let Some(op) = self.ops.get(rootref) {
            match &op.0 {
                &Operation::Const(val) => format!("{}", val),
                &Operation::Is(ref cond) => format!("{}", cond),
                &Operation::In(ref cond) => format!("{}", cond),
                &Operation::Not(opref) => format!("!({})", self.display(Some(opref))),
                &Operation::Or(lhs, rhs) => format!(
                    "({0} || {1})",
                    self.display(Some(lhs)),
                    self.display(Some(rhs))
                ),
                &Operation::And(lhs, rhs) => format!(
                    "({0} && {1})",
                    self.display(Some(lhs)),
                    self.display(Some(rhs))
                ),
            }
        } else {
            format!("<badref: {0}/{1}>", rootref, last)
        }
    }

    fn last(&self) -> Result<OpRef> {
        if !self.ops.is_empty() {
            Ok(self.ops.len() - 1)
        } else {
            Err(Error::ExpressionOutOfBounds(0, 0, self.display(None)))
        }
    }

    fn valid(&self, op: OpRef) -> Result<OpRef> {
        if op < self.ops.len() {
            Ok(op)
        } else {
            Err(Error::ExpressionOutOfBounds(
                op,
                self.last()?,
                self.display(None),
            ))
        }
    }

    pub fn new() -> Expression<Pid> {
        Expression { ops: Vec::new() }
    }

    pub fn constant(&mut self, value: bool) -> Result<OpRef> {
        self.ops.push((Operation::Const(value), 0));
        Ok(self.last()?)
    }

    pub fn is(&mut self, variable: Pid, value: Value) -> Result<OpRef> {
        let cond = Is::new(variable, value)?;
        self.ops.push((Operation::Is(cond), 0));
        Ok(self.last()?)
    }

    pub fn is_in<I>(&mut self, variable: Pid, values: I) -> Result<OpRef>
    where
        I: IntoIterator<Item = Value>,
    {
        let cond = In::new(variable, values.into_iter().collect())?;
        self.ops.push((Operation::In(cond), 0));
        Ok(self.last()?)
    }

    pub fn not(&mut self, opref: OpRef) -> Result<OpRef> {
        self.ops.push((Operation::Not(self.valid(opref)?), 0));
        self.ops[opref].1 += 1;
        Ok(self.last()?)
    }

    pub fn or(&mut self, lhs: OpRef, rhs: OpRef) -> Result<OpRef> {
        self.ops
            .push((Operation::Or(self.valid(lhs)?, self.valid(rhs)?), 0));
        self.ops[lhs].1 += 1;
        self.ops[rhs].1 += 1;
        Ok(self.last()?)
    }

    pub fn and(&mut self, lhs: OpRef, rhs: OpRef) -> Result<OpRef> {
        self.ops
            .push((Operation::And(self.valid(lhs)?, self.valid(rhs)?), 0));
        self.ops[lhs].1 += 1;
        self.ops[rhs].1 += 1;
        Ok(self.last()?)
    }

    pub fn variables(&self) -> Context<Pid> {
        Context::request(self.ops.iter().filter_map(|op| match &op.0 {
            &Operation::Is(ref cond) => Some(cond.variable()),
            &Operation::In(ref cond) => Some(cond.variable()),
            _ => None,
        }))
    }

    fn eval_single(
        &self,
        idx: OpRef,
        op: &Operation<Pid>,
        results: &Operations<Pid>,
        context: &Context<Pid>,
    ) -> Result<Option<bool>> {
        match op {
            &Operation::Is(ref cond) => {
                if let Some(val) = context.value(cond.variable()) {
                    cond.eval(val).map(|res| Some(res))
                } else {
                    Ok(None)
                }
            }
            &Operation::In(ref cond) => {
                if let Some(val) = context.value(cond.variable()) {
                    cond.eval(val).map(|res| Some(res))
                } else {
                    Ok(None)
                }
            }
            &Operation::Not(opref) => {
                let deref = results.get(opref).ok_or(Error::ExpressionFutureReference(
                    opref,
                    idx,
                    self.display(Some(idx)),
                ))?;

                if let Operation::Const(val) = deref.0 {
                    Ok(Some(!val))
                } else {
                    Ok(None)
                }
            }
            &Operation::Or(lhs, rhs) => {
                let lop = results.get(lhs).ok_or(Error::ExpressionFutureReference(
                    lhs,
                    idx,
                    self.display(Some(idx)),
                ))?;
                let rop = results.get(rhs).ok_or(Error::ExpressionFutureReference(
                    rhs,
                    idx,
                    self.display(Some(idx)),
                ))?;

                if let (Operation::Const(lval), Operation::Const(rval)) = (&lop.0, &rop.0) {
                    Ok(Some(*lval || *rval))
                } else {
                    Ok(None)
                }
            }
            &Operation::And(lhs, rhs) => {
                let lop = results.get(lhs).ok_or(Error::ExpressionFutureReference(
                    lhs,
                    idx,
                    self.display(Some(idx)),
                ))?;
                let rop = results.get(rhs).ok_or(Error::ExpressionFutureReference(
                    rhs,
                    idx,
                    self.display(Some(idx)),
                ))?;

                if let (Operation::Const(lval), Operation::Const(rval)) = (&lop.0, &rop.0) {
                    Ok(Some(*lval && *rval))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub fn eval(&self, context: &Context<Pid>) -> Result<Evaluated<Pid>> {
        if self.ops.is_empty() {
            return Err(Error::ExpressionNoop);
        }

        let mut partial: Operations<Pid> = Vec::with_capacity(self.ops.len());

        for (idx, op) in self.ops.iter().enumerate() {
            if op.1 <= 0 && idx != self.ops.len() - 1 {
                return Err(Error::ExpressionDisconnected(
                    idx,
                    self.display(Some(idx)),
                    self.display(None),
                ));
            }

            partial.push(match self.eval_single(idx, &op.0, &partial, context)? {
                Some(val) => (Operation::Const(val), op.1),
                None => op.clone(),
            });
        }

        if let Some((Operation::Const(result), _)) = partial.get(self.last()?) {
            Ok(Evaluated::Fully(*result, partial))
        } else {
            Ok(Evaluated::Partially(Expression { ops: partial }))
        }
    }
}

impl<Pid: Property> Display for Expression<Pid> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.display(None))
    }
}

impl<Pid: Property> Default for Expression<Pid> {
    fn default() -> Self {
        Expression::new()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::testproperty::Property;
    use crate::value::Datatype;

    #[test]
    fn context_request() {
        let props = vec![Property::Str, Property::Bool];
        let context = Context::request(props.clone());

        let expected = props.iter().collect::<HashSet<_>>();
        let actual = context.requested().collect::<HashSet<_>>();
        assert_eq!(expected, actual);
    }

    #[test]
    fn context_provide_known() {
        let mut context = Context::request(vec![Property::Str]);
        let res = context.provide(Property::Str, Value::Str("known".to_owned()));

        assert!(matches!(res, Ok(())));

        let provided = context.provided().collect::<HashSet<_>>();
        assert!(provided.contains(&Property::Str));

        let expected = Value::Str("known".to_owned());
        assert_eq!(context.value(Property::Str), Some(&expected));
    }

    #[test]
    fn context_provide_unknown() {
        let mut context = Context::request(vec![Property::Bool]);
        let res = context.provide(Property::Int, Value::Int(42));

        assert!(matches!(res, Ok(())));

        let provided = context.provided().collect::<HashSet<_>>();
        assert!(!provided.contains(&Property::Int));
        assert!(matches!(context.value(Property::Int), None));
    }

    #[test]
    fn context_provide_type_mismatch() {
        let mut context = Context::request(vec![Property::Bool]);
        let res = context.provide(Property::Bool, Value::Int(42));

        assert!(matches!(
            res,
            Err(Error::TypeMismatch(
                "Property::Bool",
                Datatype::Bool,
                Datatype::Int
            ))
        ));
    }

    #[test]
    fn context_provide_update() {
        let mut context = Context::request(vec![Property::Str]);
        context
            .provide(Property::Str, Value::Str("initial".to_owned()))
            .unwrap();
        assert!(
            matches!(context.value(Property::Str), Some(Value::Str(x)) if x == "initial"),
            "{}",
            context
        );

        context
            .provide(Property::Str, Value::Str("updated".to_owned()))
            .unwrap();
        assert!(
            matches!(context.value(Property::Str), Some(Value::Str(x)) if x == "updated"),
            "{}",
            context
        );
    }

    #[test]
    fn expression_variables() {
        let mut expr = Expression::<Property>::new();

        let a = expr.constant(true).unwrap();
        let b = expr.is(Property::Int, Value::Int(42)).unwrap();
        let c = expr
            .is_in(
                Property::Str,
                vec![
                    Value::Str("a match".to_owned()),
                    Value::Str("another match".to_owned()),
                ],
            )
            .unwrap();

        let a_and_b = expr.and(a, b).unwrap();
        let _root = expr.or(a_and_b, c);

        let requested = expr
            .variables()
            .requested()
            .map(|op| op.clone())
            .collect::<HashSet<_>>();
        let expected = vec![Property::Int, Property::Str]
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(requested, expected);
    }

    #[test]
    fn expression_eval_success_no_variables() {
        let mut expr = Expression::<Property>::new();

        let a = expr.constant(true).unwrap();
        let b = expr.constant(false).unwrap();
        let aorb = expr.or(a, b).unwrap();
        let _ = expr.and(aorb, b);

        let result = expr.eval(&Context::empty());
        let evaluated = result.unwrap();

        assert!(
            matches!(evaluated, Evaluated::Fully(x, _) if x == false),
            "{:?}",
            evaluated
        );
    }

    #[test]
    fn expression_eval_success_variables() {
        let mut expr = Expression::<Property>::new();

        let a = expr.is(Property::Bool, Value::Bool(true)).unwrap();
        let b = expr
            .is_in(Property::Int, vec![Value::Int(41), Value::Int(42)])
            .unwrap();

        let _ = expr.and(a, b).unwrap();
        let mut context = expr.variables();

        // #1: a is true, b is false
        context.provide(Property::Bool, Value::Bool(true)).unwrap();
        context.provide(Property::Int, Value::Int(99)).unwrap();

        let a_not_b = expr.eval(&context);
        assert!(matches!(a_not_b, Ok(Evaluated::Fully(x, _)) if x == false));

        // #2: a is false, b is true
        context.provide(Property::Bool, Value::Bool(false)).unwrap();
        context.provide(Property::Int, Value::Int(41)).unwrap();

        let b_not_a = expr.eval(&context);
        assert!(matches!(b_not_a, Ok(Evaluated::Fully(x, _)) if x == false));

        // #3: a is true, b is true
        context.provide(Property::Bool, Value::Bool(true)).unwrap();
        context.provide(Property::Int, Value::Int(42)).unwrap();

        let a_b = expr.eval(&context);
        assert!(
            matches!(a_b, Ok(Evaluated::Fully(x, _)) if x == true),
            "result: '{}', expression: '{}', context: '{}'",
            a_b.unwrap(),
            expr,
            context
        );
    }

    // Tests TODO:
    // builder methods: all failures (outofbounds)
    // partial calcaulated
    // continue partial calculation
    // eval failures: disconnected, future reference
}
