use crate::state::content_store::model::{FileChangeJournal, FileState};

pub(in crate::state::content_store) fn journal_move(
    journal: &FileChangeJournal,
) -> Option<(&FileState, &FileState)> {
    let before = journal.before.as_ref()?;
    let after = journal.after.as_ref()?;
    (before.present
        && after.present
        && before.sha512 == after.sha512
        && before.storage_kind == after.storage_kind
        && before.relative_path != after.relative_path)
        .then_some((before, after))
}

pub(in crate::state::content_store) fn journal_noop(
    journal: &FileChangeJournal,
) -> bool {
    matches!(
        (&journal.before, &journal.after),
        (Some(before), Some(after))
            if before == after
    )
}
