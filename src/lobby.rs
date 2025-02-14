use crate::messages::{ClientActorMessage, Connect, Disconnect, GetOrCreateGroup, NotifyPollId, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;


type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>, //self id to self
    rooms: HashMap<Uuid, HashSet<Uuid>>,      //room id  to list of users id
    poll_to_group: HashMap<i64, Uuid>, //poll id to group id
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            poll_to_group: HashMap::new(),
        }
    }
}

impl Lobby {
    pub fn get_or_create_group(&mut self, poll_id: i64) -> Uuid {
        // If the poll_id exists, return the group_id, else create a new one
        *self.poll_to_group.entry(poll_id).or_insert_with(Uuid::new_v4)
    }
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient
                .do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.rooms
                .get(&msg.room_id)
                .unwrap()
                .iter()
                .filter(|conn_id| *conn_id.to_owned() != msg.id)
                .for_each(|user_id| self.send_message(&format!("{} disconnected.", &msg.id), user_id));
            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);
                } else {
                    //only one in the lobby, remove it entirely
                    self.rooms.remove(&msg.room_id);
                }
            }
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new).insert(msg.self_id);

        self
            .rooms
            .get(&msg.lobby_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.self_id)
            .for_each(|conn_id| self.send_message(&format!("{} just joined!", msg.self_id), conn_id));

        self.sessions.insert(
            msg.self_id,
            msg.addr,
        );

        self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap());
            }
        } else {
            self.rooms.get(&msg.room_id).unwrap().iter().for_each(|client| self.send_message(&msg.msg, client));
        }
    }
}




impl Handler<GetOrCreateGroup> for Lobby {
    type Result = Result<Uuid, ()>;

    fn handle(&mut self, msg: GetOrCreateGroup, _: &mut Context<Self>) -> Self::Result {
        // Lookup or create the group for the i64 poll_id
        let group_id = self.get_or_create_group(msg.poll_id);
        Ok(group_id)
    }
}


impl Handler<NotifyPollId> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: NotifyPollId, _: &mut Context<Self>) -> Self::Result {
        println!("Received NotifyPollId message with poll_id: {}", msg.poll_id);

        // Check if the poll_id is mapped to a group
        if let Some(group_id) = self.poll_to_group.get(&msg.poll_id) {
            println!("Poll ID {} is mapped to group {}", msg.poll_id, group_id);

            // Broadcast a notification to all clients in this group
            if let Some(clients) = self.rooms.get(group_id) {
                let notification = format!("Notification to group {}", group_id);

                clients.iter().for_each(|client_id| {
                    self.send_message(&notification, client_id);
                });

                println!("Sent notification to {} clients in group {}", clients.len(), group_id);
            } else {
                println!("No clients found for group {}", group_id);
            }
        } else {
            println!("No group found for poll_id: {}", msg.poll_id);
        }
    }
}