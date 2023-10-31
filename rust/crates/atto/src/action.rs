use crate::value::Token;
use std::fmt;

type Args<'t, 's, S> = (&'t mut Token, &'s mut S);

type ActRet<'t> = Option<&'t mut Token>;

type ActFn<S> = for<'t, 's> fn(&'t mut Token, &'s mut S) -> ActRet<'t>;

#[derive(Clone, Eq, PartialEq)]
pub struct Action<S = ()> {
  pub action_fn: ActFn<S>,
  pub name:      &'static str,
}

impl<S> fmt::Debug for Action<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}()", self.name)
  }
}

impl<S> Default for Action<S> {
  fn default() -> Self {
    Action {
      action_fn: |token, _| Some(token),
      name:      "default",
    }
  }
}

impl<'t, 's, S> Fn<Args<'t, 's, S>> for Action<S> {
  extern "rust-call" fn call(&self, a: Args<'t, 's, S>) -> ActRet<'t> {
    (self.action_fn)(a.0, a.1)
  }
}

impl<'t, 's, S> FnMut<Args<'t, 's, S>> for Action<S> {
  extern "rust-call" fn call_mut(&mut self, a: Args<'t, 's, S>) -> ActRet<'t> {
    (self.action_fn)(a.0, a.1)
  }
}

impl<'t, 's, S> FnOnce<Args<'t, 's, S>> for Action<S> {
  type Output = ActRet<'t>;

  extern "rust-call" fn call_once(self, a: Args<'t, 's, S>) -> ActRet<'t> {
    (self.action_fn)(a.0, a.1)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // The tests all run action() twice to make sure borrowing works

  #[test]
  fn test_identity() {
    let action = Action::<()>::default();
    let mut token = Token::default();
    assert_eq!(
      action(&mut token, &mut ()).map(|token| &*token),
      Some(&Token::default())
    );
    assert_eq!(
      action(&mut token, &mut ()).map(|token| &*token),
      Some(&Token::default())
    );
  }

  #[test]
  fn test_mut_state() {
    fn action_fn<'t>(token: &'t mut Token, state: &'_ mut u32) -> ActRet<'t> {
      *state += 1;
      Some(token)
    }

    let mut state = 42;
    let action = Action {
      action_fn,
      name: "test",
    };

    let mut token = Token::default();
    action(&mut token, &mut state);
    assert_eq!(state, 43);
    action(&mut token, &mut state);
    assert_eq!(state, 44);
  }

  #[test]
  fn test_mut_token() {
    fn action_fn<'t>(token: &'t mut Token, _: &'_ mut ()) -> ActRet<'t> {
      token.id += 1;
      Some(token)
    }
    let action = Action {
      action_fn,
      name: "test",
    };

    let mut token = Token::default();
    assert_eq!(1, action(&mut token, &mut ()).unwrap().id);
    assert_eq!(2, action(&mut token, &mut ()).unwrap().id);
  }

  #[test]
  fn test_with_closure_syntax() {
    let action = Action {
      // 1. type inferred from action() calls below
      // 2. closure syntax without capture gives a fn pointer
      action_fn: |token, _| {
        token.id += 1;
        Some(token)
      },
      name:      "test",
    };

    let mut token = Token::default();
    assert_eq!(1, action(&mut token, &mut ()).unwrap().id);
    assert_eq!(2, action(&mut token, &mut ()).unwrap().id);
  }
}
