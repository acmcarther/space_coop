pub use self::helpers::{
  try_recv_all
};
mod helpers {
  use std::iter::{repeat};
  use std::sync::mpsc::{Receiver};

  pub fn try_recv_all<T>(ack_rx: &Receiver<T>) -> Vec<T> {
    repeat(()).map(|_| ack_rx.try_recv().ok())
      .take_while(|x| x.is_some())
      .map(|x| x.unwrap())
      .collect()
  }

  #[cfg(test)]
  mod tests {
    use std::sync::mpsc::channel;
    use helpers::try_recv_all;

    #[test]
    fn try_recv_all_with_none() {
      let (_, rx) = channel::<()>();

      assert_eq!(try_recv_all(&rx).len(), 0);
    }

    #[test]
    fn try_recv_all_with_one() {
      let (tx, rx) = channel();
      let _ = tx.send(5);

      let result = try_recv_all(&rx);
      assert_eq!(result, vec![5]);
    }

    #[test]
    fn try_recv_all_with_many() {
      let (tx, rx) = channel();
      for x in (1..5).into_iter() {
        let _ = tx.send(x);
      }

      let result = try_recv_all(&rx);
      assert_eq!(result, vec![1, 2, 3, 4]);
    }
  }
}

