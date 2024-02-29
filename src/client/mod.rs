pub mod editor;
pub mod history;

use editor::Editor;
use history::History;

fn main() {
    let histo = History::initialize();
    Editor::read(histo);
}