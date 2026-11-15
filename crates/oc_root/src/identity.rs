use uuid::Uuid;

use crate::side::Side;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Identity {
    pub uuid: Uuid,
    pub side: Side,
}
