use cucumber::CucumberRegistrar;
use support::ClientWorld;
use common::protocol::{ClientEvent, ClientNetworkEvent};
use pubsub::PubSubStore;
use std::ops::Deref;
use std::str::FromStr;
use itertools::Itertools;

pub fn register_steps(c: &mut CucumberRegistrar<ClientWorld>) {
  Given!(c,
         "^we monitor outgoing network events$",
         |_, world: &mut ClientWorld, _| {
           let sub = world.planner.mut_world().register_subscriber::<ClientNetworkEvent>();
           world.net_subscriber = Some(sub);
         });

  Then!(c,
        "^the following network events are emitted:$",
        |_, world: &mut ClientWorld, (table,): (Vec<Vec<String>>,)| {
    let token = &world.net_subscriber;
    match token {
      &Some(ref token) => {
        let events = world.planner
          .mut_world()
          .fetch_subscriber(token)
          .collected();
        assert_eq!(events.len(), table.len());
        table.into_iter().foreach(|row| {
          match row.get(0).map(|v| v.deref()) {
            None => panic!("need an event type as first item column"),
            Some("DomainEvent") => check_for_domain_event(&events, &row),
            Some(x) => panic!("unknown event type \"{}\"", x),
          }
        });
      },
      &None => panic!("call \"we monitor outgoing events\" before trying to check emitted events"),
    }
  });

  Then!(c,
        "^no network events are emitted$",
        |_, world: &mut ClientWorld, _| {
    let token = &world.net_subscriber;
    match token {
      &Some(ref token) => {
        assert_eq!(world.planner
                     .mut_world()
                     .fetch_subscriber(token)
                     .collected()
                     .len(),
                   0)
      },
      &None => {
        panic!("call \"we monitor outgoing events\" before trying to check \
                                      emitted events");
      },
    }
  });
}

fn check_for_domain_event(events: &Vec<ClientNetworkEvent>, row: &Vec<String>) {
  match row.get(1).map(|x| x.deref()) {
    None => panic!("Need domain event type as second column"),
    Some("SelfMove") => {
      match (row.get(2).and_then(|x| f32::from_str(x).ok()),
             row.get(3).and_then(|y| f32::from_str(y).ok()),
             row.get(4).and_then(|z| f32::from_str(z).ok())) {
        (Some(x), Some(y), Some(z)) => {
          check_for_exact_domain_event(events,
                                       ClientEvent::SelfMove {
                                         x_d: x,
                                         y_d: y,
                                         z_d: z,
                                       })
        },
        _ => panic!("Need numbers for third, fourth, and fifth column"),
      }
    },
    Some(x) => panic!("unknown domain event type \"{}\"", x),
  }
}

fn check_for_exact_domain_event(events: &Vec<ClientNetworkEvent>, self_move: ClientEvent) {
  let full_event = ClientNetworkEvent::DomainEvent(self_move);
  assert!(events.iter().any(|x| *x == full_event),
          "Expect list to contain event {:?}",
          full_event);
}
