use magnus::{block::Proc, Error, Ruby, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use yrs::block::ClientID;
use yrs::sync::awareness::Event;
use yrs::sync::{Awareness, AwarenessUpdate};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, OffsetKind, Options, Subscription};

#[magnus::wrap(class = "Y::Awareness")]
pub(crate) struct YAwareness(pub(crate) RefCell<Awareness>);

/// SAFETY: This is safe because we only access this data when the GVL is held.
unsafe impl Send for YAwareness {}

impl YAwareness {
    pub(crate) fn yawareness_new() -> Self {
        let mut options = Options {
            offset_kind: OffsetKind::Utf16,
            ..Default::default()
        };
        options.offset_kind = OffsetKind::Utf16;

        let doc = Doc::with_options(options);

        let awareness = Awareness::new(doc);

        Self(RefCell::new(awareness))
    }

    pub(crate) fn yawareness_apply_update(&self, update: Vec<u8>) -> Result<(), Error> {
        let ruby = Ruby::get().unwrap();
        AwarenessUpdate::decode_v1(update.as_slice())
            .map_err(|_error| Error::new(ruby.exception_runtime_error(), "cannot decode update"))
            .and_then(|value| {
                self.0.borrow_mut().apply_update(value).map_err(|_error| {
                    Error::new(
                        ruby.exception_runtime_error(),
                        "cannot apply awareness update",
                    )
                })
            })
    }

    pub(crate) fn yawareness_clean_local_state(&self) {
        self.0.borrow_mut().clean_local_state();
    }

    pub(crate) fn yawareness_client_id(&self) -> u64 {
        self.0.borrow().client_id().get()
    }

    pub(crate) fn yawareness_clients(&self) -> HashMap<u64, String> {
        self.0
            .borrow()
            .iter()
            .filter_map(|(client_id, state)| {
                state.data.map(|data| (client_id.get(), data.to_string()))
            })
            .collect()
    }

    pub(crate) fn yawareness_local_state(&self) -> Option<String> {
        self.0
            .borrow()
            .local_state_raw()
            .map(|value| value.to_string())
    }

    pub(crate) fn yawareness_on_update(&self, block: Proc) -> YAwarenessSubscription {
        let subscription = self
            .0
            .borrow_mut()
            .on_update(move |_awareness, event, _origin| {
                let awareness_event = YAwarenessEvent::from(event);
                let args = (awareness_event,);
                block
                    .call::<(YAwarenessEvent,), Value>(args)
                    .expect("cannot call block: on_update");
            });

        // we need to make sure the event handler "survives" and is not being
        // dropped after leaving this scope, so we pass it back to Ruby.
        YAwarenessSubscription::from(subscription)
    }

    pub(crate) fn yawareness_remove_state(&self, client_id: u64) {
        self.0.borrow_mut().remove_state(ClientID::new(client_id))
    }

    pub(crate) fn yawareness_set_local_state(&self, json: String) {
        self.0.borrow_mut().set_local_state_raw(json)
    }

    pub(crate) fn yawareness_update(&self) -> Result<Vec<u8>, Error> {
        let ruby = Ruby::get().unwrap();
        self.0
            .borrow_mut()
            .update()
            .map(|update| update.encode_v1())
            .map_err(|_error| {
                Error::new(
                    ruby.exception_runtime_error(),
                    "cannot create update for current state",
                )
            })
    }

    pub(crate) fn yawareness_update_with_clients(
        &self,
        clients: Vec<u64>,
    ) -> Result<Vec<u8>, Error> {
        let ruby = Ruby::get().unwrap();
        let clients = clients.into_iter().map(ClientID::new);
        self.0
            .borrow_mut()
            .update_with_clients(clients)
            .map(|update| update.encode_v1())
            .map_err(|_error| {
                Error::new(
                    ruby.exception_runtime_error(),
                    "cannot create update for current state and given clients",
                )
            })
    }
}

impl From<Awareness> for YAwareness {
    fn from(value: Awareness) -> Self {
        Self(RefCell::from(value))
    }
}

#[magnus::wrap(class = "Y::AwarenessEvent")]
pub(crate) struct YAwarenessEvent(Event);

/// SAFETY: This is safe because we only access this data when the GVL is held.
unsafe impl Send for YAwarenessEvent {}

impl YAwarenessEvent {
    pub(crate) fn added(&self) -> Vec<u64> {
        self.0.added().iter().map(|id| id.get()).collect()
    }
    pub(crate) fn updated(&self) -> Vec<u64> {
        self.0.updated().iter().map(|id| id.get()).collect()
    }
    pub(crate) fn removed(&self) -> Vec<u64> {
        self.0.removed().iter().map(|id| id.get()).collect()
    }
}

impl From<&Event> for YAwarenessEvent {
    fn from(value: &Event) -> Self {
        Self(value.clone())
    }
}

#[allow(dead_code)]
#[magnus::wrap(class = "Y::AwarenessEvent")]
pub(crate) struct YAwarenessSubscription(Subscription);

unsafe impl Send for YAwarenessSubscription {}

impl YAwarenessSubscription {}

impl From<Subscription> for YAwarenessSubscription {
    fn from(v: Subscription) -> Self {
        YAwarenessSubscription(v)
    }
}
