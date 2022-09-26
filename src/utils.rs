#[derive(Debug, Clone)]
pub enum Either<A, B> {
  Left(A),
  Right(B)
}

impl<A, B> Either<A, B> {
  pub fn is_left(&self) -> bool {
    match self {
      Either::Left(_) => true,
      Either::Right(_) => false
    }
  }
  pub fn is_right(&self) -> bool {
    match self {
      Either::Left(_) => false,
      Either::Right(_) => true
    }
  }

  pub fn get_left(self) -> Option<A> {
    match self {
      Either::Left(a) => Some(a),
      Either::Right(_) => None
    }
  }
  pub fn get_right(self) -> Option<B> {
    match self {
      Either::Left(_) => None,
      Either::Right(b) => Some(b)
    }
  }

  pub fn get_left_with_default(self, default: A) -> A {
    match self {
      Either::Left(a) => a,
      Either::Right(_) => default
    }
  }
  pub fn get_right_with_default(self, default: B) -> B {
    match self {
      Either::Left(_) => default,
      Either::Right(b) => b
    }
  }
}