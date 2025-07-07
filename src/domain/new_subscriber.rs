use crate::domain::subscriber_name::SubscriberName;
use email_address::EmailAddress;

pub struct NewSubscriber {
    pub email: EmailAddress,
    pub name: SubscriberName,
}
