use crate::{
    data::{Directory, Note},
    db::Db,
    state::notebook::{DirectoryItem, InnerState, NotebookState, SelectedItem},
    Error, NotebookTransition, Result,
};

pub fn show_actions_dialog(state: &mut NotebookState, note: Note) -> Result<NotebookTransition> {
    state.inner_state = InnerState::NoteMoreActions;

    Ok(NotebookTransition::ShowNoteActionsDialog(note))
}

pub fn select(state: &mut NotebookState, note: Note) -> Result<NotebookTransition> {
    state.selected = SelectedItem::Note(note);
    state.inner_state = InnerState::NoteSelected;

    Ok(NotebookTransition::None)
}

pub async fn rename(
    db: &mut Db,
    state: &mut NotebookState,
    mut note: Note,
    new_name: String,
) -> Result<NotebookTransition> {
    db.rename_note(note.id.clone(), new_name.clone()).await?;

    note.name = new_name;
    state.selected = SelectedItem::Note(note.clone());
    state.inner_state = InnerState::NoteSelected;

    Ok(NotebookTransition::RenameNote(note))
}

pub async fn remove(
    db: &mut Db,
    state: &mut NotebookState,
    note: Note,
) -> Result<NotebookTransition> {
    db.remove_note(note.id.clone()).await?;

    // TODO
    state.selected = SelectedItem::None;

    Ok(NotebookTransition::RemoveNote(note))
}

pub async fn add(
    db: &mut Db,
    state: &mut NotebookState,
    directory: Directory,
    note_name: String,
) -> Result<NotebookTransition> {
    let note = db.add_note(directory.id.clone(), note_name).await?;

    let item = state
        .root
        .find_mut(&directory.id)
        .ok_or(Error::Wip("todo: failed to find".to_owned()))?;

    if let DirectoryItem {
        children: Some(ref mut children),
        ..
    } = item
    {
        let notes = db.fetch_notes(directory.id.clone()).await?;
        children.notes = notes;
    }

    state.selected = SelectedItem::Note(note.clone());
    state.inner_state = InnerState::NoteSelected;

    Ok(NotebookTransition::AddNote(note))
}

pub async fn open(
    db: &mut Db,
    state: &mut NotebookState,
    note: Note,
) -> Result<NotebookTransition> {
    let content = db.fetch_note_content(note.id.clone()).await?;

    state.editing = Some(note.clone());
    state.inner_state = InnerState::EditingViewMode;

    Ok(NotebookTransition::OpenNote { note, content })
}

pub async fn edit(state: &mut NotebookState) -> Result<NotebookTransition> {
    state.inner_state = InnerState::EditingEditMode;

    Ok(NotebookTransition::EditMode)
}

pub async fn view(state: &mut NotebookState) -> Result<NotebookTransition> {
    let note = state.get_editing()?.clone();

    state.inner_state = InnerState::EditingViewMode;

    Ok(NotebookTransition::ViewMode(note))
}

pub async fn browse(state: &mut NotebookState) -> Result<NotebookTransition> {
    let note = state.get_selected_note()?.clone();

    state.inner_state = InnerState::NoteSelected;

    Ok(NotebookTransition::SelectNote(note))
}

pub async fn update_content(
    db: &mut Db,
    state: &mut NotebookState,
    content: String,
) -> Result<NotebookTransition> {
    let id = state.get_editing()?.id.clone();

    db.update_note_content(id, content).await?;

    Ok(NotebookTransition::UpdateNoteContent)
}
