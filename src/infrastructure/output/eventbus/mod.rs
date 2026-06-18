pub mod room;
pub mod activity;
pub mod game;
pub mod chat;

#[macro_export]
macro_rules! impl_tokio_event_bus {
    ($struct_name:ident, $sender_type:ty, $receiver_type:ty, $event_type:ty) => {
        pub struct $struct_name($sender_type);

        impl $struct_name {
            pub fn new(capacity: usize) -> Self {
                Self(<$sender_type>::new(capacity))
            }
        }

        impl crate::domain::EventBus<$event_type> for $struct_name {
            fn subscribe(&self) -> $receiver_type {
                self.0.subscribe()
            }

            fn publish(&self, event: $event_type) {
                tracing::info!("get published");
                let _ = self.0.send(event);
            }
        }
    };
}
pub(super) use impl_tokio_event_bus;