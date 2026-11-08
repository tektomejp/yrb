use magnus::{Error, Ruby};
use yrs::updates::decoder::Decode;
use yrs::{IdSet, Update};

#[magnus::wrap(class = "Y::Update")]
pub(crate) struct YUpdate {
    insertions: Vec<(u64, u32, u32)>,
    deletions: Vec<(u64, u32, u32)>,
}

impl From<Update> for YUpdate {
    fn from(update: Update) -> Self {
        Self {
            insertions: clock_ranges(&update.insertions(true)),
            deletions: clock_ranges(update.delete_set()),
        }
    }
}

impl YUpdate {
    pub(crate) fn new(bytes: Vec<u8>) -> Result<Self, Error> {
        Update::decode_v1(&bytes).map(Self::from).map_err(|error| {
            Error::new(
                Ruby::get().unwrap().exception_runtime_error(),
                format!("cannot decode update: {:?}", error),
            )
        })
    }

    pub(crate) fn insertions(&self) -> Vec<(u64, u32, u32)> {
        self.insertions.clone()
    }

    pub(crate) fn deletions(&self) -> Vec<(u64, u32, u32)> {
        self.deletions.clone()
    }
}

fn clock_ranges(ids: &IdSet) -> Vec<(u64, u32, u32)> {
    ids.iter()
        .flat_map(|(client, ranges)| {
            ranges
                .iter()
                .map(move |range| (client.get(), range.start, range.end))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yrs::updates::encoder::Encode;
    use yrs::{Doc, GetString, ReadTxn, StateVector, Text, Transact};

    #[test]
    fn nonzero_clock_insertions_are_not_lost() {
        let doc = Doc::with_client_id(17);
        let text = doc.get_or_insert_text("body");
        text.push(&mut doc.transact_mut(), "Door ");
        let before = doc.transact().state_vector();
        text.push(&mut doc.transact_mut(), "850");
        let bytes = doc.transact().encode_diff_v1(&before);
        let update = YUpdate::from(Update::decode_v1(&bytes).unwrap());
        assert_eq!(update.insertions(), vec![(17, 5, 8)]);
        assert!(update.deletions().is_empty());
    }

    #[test]
    fn deletion_ranges_are_distinct_from_insertions() {
        let doc = Doc::with_client_id(17);
        let text = doc.get_or_insert_text("body");
        text.push(&mut doc.transact_mut(), "Door 850");
        let before = doc.transact().state_vector();
        text.remove_range(&mut doc.transact_mut(), 5, 3);
        let bytes = doc.transact().encode_diff_v1(&before);
        let update = YUpdate::from(Update::decode_v1(&bytes).unwrap());
        assert!(update.insertions().is_empty());
        assert_eq!(update.deletions(), vec![(17, 5, 8)]);
    }

    #[test]
    fn separated_ranges_and_multiple_clients_keep_their_exact_clocks() {
        let source = Doc::with_client_id(17);
        let text = source.get_or_insert_text("body");
        text.push(&mut source.transact_mut(), "a");
        let first =
            Update::decode_v1(&source.transact().encode_diff_v1(&StateVector::default())).unwrap();
        text.push(&mut source.transact_mut(), "b");
        let before = source.transact().state_vector();
        text.push(&mut source.transact_mut(), "c");
        let third = Update::decode_v1(&source.transact().encode_diff_v1(&before)).unwrap();
        let other = Doc::with_client_id(9);
        other
            .get_or_insert_text("other")
            .push(&mut other.transact_mut(), "x");
        let other =
            Update::decode_v1(&other.transact().encode_diff_v1(&StateVector::default())).unwrap();
        let update = Update::merge_updates([first, third, other]);
        assert_eq!(
            YUpdate::from(update).insertions(),
            vec![(9, 0, 1), (17, 0, 1), (17, 2, 3)]
        );
    }

    #[test]
    fn complete_encoding_keeps_a_waiting_deletion() {
        let source = Doc::with_client_id(17);
        let text = source.get_or_insert_text("body");
        text.push(&mut source.transact_mut(), "Door 850");
        let insertion = source
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        text.remove_range(&mut source.transact_mut(), 5, 3);
        let deletion = source.transact().snapshot().delete_set;
        let mut bytes = vec![0];
        bytes.extend(deletion.encode_v1());
        let waiting = Doc::new();
        waiting
            .transact_mut()
            .apply_update(Update::decode_v1(&bytes).unwrap())
            .unwrap();
        assert!(waiting.transact().has_missing_updates());
        let encoded = waiting
            .transact()
            .encode_state_as_update_v1(&StateVector::default());
        let restored = Doc::new();
        restored
            .transact_mut()
            .apply_update(Update::decode_v1(&encoded).unwrap())
            .unwrap();
        restored
            .transact_mut()
            .apply_update(Update::decode_v1(&insertion).unwrap())
            .unwrap();
        assert_eq!(
            restored
                .get_or_insert_text("body")
                .get_string(&restored.transact()),
            "Door "
        );
        assert!(!restored.transact().has_missing_updates());
    }
}
