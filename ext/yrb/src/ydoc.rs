use crate::utils::next_subscription_id;
use crate::yarray::YArray;
use crate::ymap::YMap;
use crate::ytext::YText;
use crate::yxml_element::YXmlElement;
use crate::yxml_fragment::YXmlFragment;
use crate::yxml_text::YXmlText;
use crate::YTransaction;
use magnus::block::Proc;
use magnus::{Error, Integer, RArray, Ruby, Value};
use std::borrow::Borrow;
use std::cell::RefCell;
use yrs::block::ClientID;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::{Encoder, EncoderV2};
use yrs::{Doc, OffsetKind, Options, ReadTxn, StateVector, Transact};

#[magnus::wrap(class = "Y::Doc")]
pub(crate) struct YDoc(pub(crate) RefCell<Doc>);

unsafe impl Send for YDoc {}

impl YDoc {
    pub(crate) fn ydoc_new(client_id: &[Value]) -> Self {
        let mut options = Options::default();
        if client_id.len() == 1 {
            let value = client_id.first().unwrap();
            options.client_id =
                ClientID::new(Integer::from_value(*value).unwrap().to_u64().unwrap());
        }
        options.offset_kind = OffsetKind::Utf16;

        let doc = Doc::with_options(options);
        Self(RefCell::new(doc))
    }

    pub(crate) fn ydoc_encode_diff_v1(
        &self,
        transaction: &YTransaction,
        state_vector: Vec<u8>,
    ) -> Result<Vec<u8>, Error> {
        let ruby = Ruby::get().unwrap();
        let mut tx = transaction.transaction();
        let tx = tx.as_mut().unwrap();

        StateVector::decode_v1(state_vector.borrow())
            .map(|sv| tx.encode_diff_v1(&sv))
            .map_err(|_e| Error::new(ruby.exception_runtime_error(), "cannot encode diff"))
    }

    pub(crate) fn ydoc_encode_diff_v2(
        &self,
        transaction: &YTransaction,
        state_vector: Vec<u8>,
    ) -> Result<Vec<u8>, Error> {
        let ruby = Ruby::get().unwrap();
        let mut tx = transaction.transaction();
        let tx = tx.as_mut().unwrap();
        let mut encoder = EncoderV2::new();

        StateVector::decode_v2(state_vector.borrow())
            .map(|sv| tx.encode_diff(&sv, &mut encoder))
            .map(|_| encoder.to_vec())
            .map_err(|_e| Error::new(ruby.exception_runtime_error(), "cannot encode diff"))
    }

    pub(crate) fn ydoc_get_or_insert_array(&self, name: String) -> YArray {
        let array_ref = self.0.borrow().get_or_insert_array(name.as_str());
        YArray::from(array_ref)
    }

    pub(crate) fn ydoc_get_or_insert_map(&self, name: String) -> YMap {
        let map_ref = self.0.borrow().get_or_insert_map(name.as_str());
        YMap::from(map_ref)
    }

    pub(crate) fn ydoc_get_or_insert_text(&self, name: String) -> YText {
        let text_ref = self.0.borrow().get_or_insert_text(name.as_str());
        YText::from(text_ref)
    }

    // yrs >= 0.18 dropped `RootRef` for `XmlElementRef`/`XmlTextRef`: only `XmlFragmentRef` may
    // be a document root now (matching yjs, where `Y.XmlElement`/`Y.XmlText` were never valid
    // top-level shared types). There is no compatible root-level `XmlElementRef` to hand back
    // here any more, so we fail loudly instead of inventing a new, wire-incompatible encoding
    // for what used to be a root `Y::XMLElement`. Callers should migrate to `get_xml_fragment`.
    pub(crate) fn ydoc_get_or_insert_xml_element(
        &self,
        _name: String,
    ) -> Result<YXmlElement, Error> {
        let ruby = Ruby::get().unwrap();
        Err(Error::new(
            ruby.exception_runtime_error(),
            "root-level Y::XMLElement is no longer supported (yrs >= 0.18 only allows XMLFragment as a document root); use Y::Doc#get_xml_fragment instead",
        ))
    }

    pub(crate) fn ydoc_get_or_insert_xml_fragment(&self, name: String) -> YXmlFragment {
        let xml_fragment_ref = self.0.borrow().get_or_insert_xml_fragment(name.as_str());
        YXmlFragment::from(xml_fragment_ref)
    }

    // See `ydoc_get_or_insert_xml_element` above: root-level `Y::XMLText` has no yrs >= 0.18
    // equivalent either.
    pub(crate) fn ydoc_get_or_insert_xml_text(&self, _name: String) -> Result<YXmlText, Error> {
        let ruby = Ruby::get().unwrap();
        Err(Error::new(
            ruby.exception_runtime_error(),
            "root-level Y::XMLText is no longer supported (yrs >= 0.18 only allows XMLFragment as a document root); use Y::Doc#get_xml_fragment instead",
        ))
    }

    pub(crate) fn ydoc_transact(&self) -> YTransaction {
        let doc = self.0.borrow();
        let transaction = doc.transact_mut();
        YTransaction::from(transaction)
    }

    pub(crate) fn ydoc_observe_update(&self, block: Proc) -> Result<u32, Error> {
        let ruby = Ruby::get().unwrap();
        let subscription_id = next_subscription_id();
        self.0
            .borrow()
            .observe_update_v1_with(subscription_id, move |_tx, update_event| {
                let ruby = unsafe { Ruby::get_unchecked() };
                let update = update_event.update.to_vec();
                let update = ruby.ary_from_vec(update);

                let args: (RArray,) = (update,);
                block
                    .call::<(RArray,), Value>(args)
                    .expect("cannot call update block");
            })
            .map(|()| subscription_id)
            .map_err(|err| Error::new(ruby.exception_runtime_error(), err.to_string()))
    }
}
