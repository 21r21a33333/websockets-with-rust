use actix::prelude::{Message, Recipient};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub lobby_id: Uuid,
    pub self_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
    pub room_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: String,
    pub room_id: Uuid
}


#[derive(Message)]
#[rtype(result = "Result<Uuid, ()>")]
pub struct GetOrCreateGroup {
    pub poll_id: i64, // poll_id as i64
}



#[derive(Message)]
#[rtype(result = "()")]
pub struct NotifyPollId {
    pub poll_id: i64,
}