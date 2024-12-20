use std::{collections::VecDeque, str::Lines, sync::atomic::AtomicU16};

use egui::{
    text::CCursorRange, Align, Context, Event, EventFilter, Id, Key, Modifiers, TextEdit, Ui,
};

static SEARCH_PROMPT: &str = "(reverse-i-search) :";
const SEARCH_PROMPT_SLOT_OFF: usize = 18;
static INSTANCE_COUNT: AtomicU16 = AtomicU16::new(0);

/// The event that was generated by the console
///
///
pub enum ConsoleEvent {
    /// A command was entered
    Command(String),

    /// Nothing
    None,
}
/// Console Window  
///
///
#[derive(Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsoleWindow {
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) text: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) force_cursor_to_end: bool,
    history_size: usize,
    pub(crate) scrollback_size: usize,
    command_history: VecDeque<String>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    history_cursor: Option<usize>,
    pub(crate) prompt: String,
    prompt_len: usize,
    /// The id of the console window.
    pub id: Id,
    save_prompt: Option<String>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    search_partial: Option<String>,
    // enable running stuff after serde reload
    #[cfg_attr(feature = "persistence", serde(skip))]
    init_done: bool,

    // tab completion
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) tab_string: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) tab_nth: usize,
    pub(crate) tab_quote: char,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) tab_quoted: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    pub(crate) tab_offset: usize,
    pub(crate) tab_command_table: Vec<String>,
}

impl ConsoleWindow {
    pub(crate) fn new(prompt: &str) -> Self {
        Self {
            text: String::new(),
            force_cursor_to_end: false,
            command_history: VecDeque::new(),
            history_cursor: None,
            history_size: 100,
            scrollback_size: 1000,
            prompt: prompt.to_string(),
            prompt_len: prompt.chars().count(),
            id: Id::new(format!(
                "console_text_{}",
                INSTANCE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            )),
            save_prompt: None,
            search_partial: None,
            init_done: false,

            tab_string: String::new(),
            tab_nth: 0,
            tab_quote: '"',
            tab_quoted: false,
            tab_offset: usize::MAX,
            tab_command_table: Vec::new(),
        }
    }
    /// Draw the console window
    /// # Arguments
    /// * `ui` - the egui Ui context
    ///
    /// # Returns
    /// * `ConsoleEvent` - the event that was generated by the console
    ///
    pub fn draw(&mut self, ui: &mut Ui) -> ConsoleEvent {
        if !self.init_done {
            self.init_done = true;
            if let Some(prompt) = &self.save_prompt {
                self.prompt.clone_from(prompt);
                self.save_prompt = None;
            }
            self.draw_prompt();
        }
        // do we need to handle keyboard events?
        let msg = if ui.ctx().memory(|mem| mem.has_focus(self.id)) {
            self.handle_kb(ui.ctx())
        } else {
            ConsoleEvent::None
        };
        {
            let text_len = self.text.len();
            self.ui(ui);

            // did somebody type?
            if self.text.len() != text_len {
                // yes - need to update partial search?
                if self.search_partial.is_some() {
                    self.search_partial = Some(self.get_search_text().to_string());
                    self.prompt = SEARCH_PROMPT.to_string();
                    self.prompt.insert_str(
                        SEARCH_PROMPT_SLOT_OFF + 1,
                        self.search_partial.as_ref().unwrap(),
                    );
                    self.history_cursor = None;
                    self.history_back();
                }
                self.tab_string.clear();
                self.tab_nth = 0;
            }
        }

        // this is all so that we get the escape key (to exit search)
        let event_filter = EventFilter {
            escape: true,
            horizontal_arrows: true,
            vertical_arrows: true,
            tab: true, // we need the tab key for tab completion
        };
        if ui.ctx().memory(|mem| mem.has_focus(self.id)) {
            ui.ctx()
                .memory_mut(|mem| mem.set_focus_lock_filter(self.id, event_filter));
        }

        msg
    }
    /// Write a line to the console
    /// # Arguments
    /// * `data` - the string to write
    ///
    /// Note that you can call this without the user having typed anything.
    ///
    pub fn write(&mut self, data: &str) {
        self.text.push_str(&format!("\n{}", data));
        self.truncate_scroll_back();
        self.force_cursor_to_end = true;
    }

    /// Loads the history from an iterator of strings
    /// # Arguments
    /// * `history` - an iterator of strings
    ///
    ///
    pub fn load_history(&mut self, history: Lines<'_>) {
        self.command_history = history.into_iter().map(|s| s.to_string()).collect();
        self.history_cursor = None;
    }

    /// Get the history of the console
    /// # Returns
    /// * `VecDeque<String>` - the history of the console
    ///
    ///     
    pub fn get_history(&self) -> VecDeque<String> {
        self.command_history.clone()
    }
    /// Clear the history of the console
    ///
    pub fn clear_history(&mut self) {
        self.command_history.clear();
        self.history_cursor = None;
    }

    /// Clear the console
    pub fn clear(&mut self) {
        self.text.clear();
        self.force_cursor_to_end = false;
    }
    /// Prompt the user for input
    pub fn prompt(&mut self) {
        self.draw_prompt();
    }
    /// get mut ref to tab completion table for commands
    pub fn command_table_mut(&mut self) -> &mut Vec<String> {
        &mut self.tab_command_table
    }

    fn cursor_at_end(&self) -> CCursorRange {
        egui::text::CCursorRange::one(egui::text::CCursor::new(self.text.chars().count()))
    }
    fn cursor_at(&self, loc: usize) -> CCursorRange {
        if loc >= self.text.chars().count() {
            return self.cursor_at_end();
        }
        egui::text::CCursorRange::one(egui::text::CCursor::new(loc))
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().show(ui, |ui| {
            ui.add_sized(ui.available_size(), |ui: &mut Ui| {
                let widget = egui::TextEdit::multiline(&mut self.text)
                    .font(egui::TextStyle::Monospace)
                    .frame(false)
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .id(self.id);
                let output = widget.show(ui);
                let mut new_cursor = None;

                // fix up cursor position
                // different logic depending on normal vs search mode
                // scroll, mouse move etc
                // cursor might not be in a good location

                match self.search_partial {
                    Some(_) => {
                        if let Some(cursor) = output.state.cursor.char_range() {
                            let last_off = self.last_line_offset();
                            if cursor.primary.index < (last_off + SEARCH_PROMPT_SLOT_OFF + 1) {
                                new_cursor =
                                    Some(self.cursor_at(last_off + SEARCH_PROMPT_SLOT_OFF + 1));
                            } else {
                                let search_text = self.get_search_text();
                                if cursor.primary.index
                                    > (last_off + SEARCH_PROMPT.len() + search_text.len())
                                {
                                    new_cursor = Some(self.cursor_at(
                                        last_off + SEARCH_PROMPT_SLOT_OFF + search_text.len() + 1,
                                    ));
                                }
                            }
                        }
                    }
                    None => {
                        if let Some(cursor) = output.state.cursor.char_range() {
                            let last_off = self.last_line_offset();
                            if cursor.primary.index < last_off + self.prompt_len - 1 {
                                new_cursor = Some(self.cursor_at_end());
                            }
                        }

                        // we need a new line (user pressed enter)
                        if self.force_cursor_to_end {
                            new_cursor = Some(self.cursor_at_end());
                            self.force_cursor_to_end = false;
                        }
                    }
                };

                if new_cursor.is_some() {
                    let text_edit_id = output.response.id;

                    if let Some(mut state) = TextEdit::load_state(ui.ctx(), text_edit_id) {
                        state.cursor.set_char_range(new_cursor);
                        state.store(ui.ctx(), text_edit_id);
                    }
                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                }
                output.response
            });
        });
    }

    pub(crate) fn get_last_line(&self) -> &str {
        self.text
            .lines()
            .last()
            .unwrap_or("")
            .strip_prefix(&self.prompt)
            .unwrap_or("")
    }
    fn truncate_scroll_back(&mut self) {
        let line_count = self.text.lines().count();
        if line_count < self.scrollback_size {
            return;
        }
        let mut scrollback = String::with_capacity(self.text.len());

        for (i, line) in self.text.lines().enumerate() {
            if i > line_count - self.scrollback_size {
                scrollback.push_str(line);
                scrollback.push('\n');
            }
        }
        self.text = scrollback;
    }
    fn get_search_text(&self) -> &str {
        let last = self.text.lines().last().unwrap_or("");
        let mut iter = last.char_indices();
        let (start, _) = iter.nth(SEARCH_PROMPT_SLOT_OFF + 1).unwrap_or((0, ' '));
        for (end, ch) in iter {
            // TODO - this will fail if the search text contains ':'
            if ch == ':' {
                return &last[start..end];
            }
        }
        ""
    }
    fn consume_key(ctx: &Context, modifiers: Modifiers, logical_key: Key) {
        ctx.input_mut(|inp| inp.consume_key(modifiers, logical_key));
    }

    fn handle_key(
        &mut self,
        key: &Key,
        modifiers: Modifiers,
        cursor: usize,
    ) -> (bool, Option<String>) {
        // return value is (consume_key, command)

        let return_value = match (modifiers, key) {
            (Modifiers::NONE, Key::ArrowDown) => {
                // down arrow only means something if we are in search mode
                if self.search_partial.is_some() {
                    self.exit_search_mode()
                };
                if let Some(mut hc) = self.history_cursor {
                    let last = self.get_last_line();
                    self.text = self.text.strip_suffix(last).unwrap_or("").to_string();
                    if hc == self.command_history.len() - 1 {
                        self.history_cursor = None;
                    } else {
                        if hc < self.command_history.len() - 1 {
                            hc += 1;
                            self.text.push_str(self.command_history[hc].as_str());
                        }
                        self.history_cursor = Some(hc);
                    }
                }
                (true, None)
            }
            (Modifiers::NONE, Key::ArrowUp) => {
                if self.command_history.is_empty() {
                    return (true, None);
                }
                if self.search_partial.is_some() {
                    self.exit_search_mode()
                };

                self.history_back();
                (true, None)
            }
            (Modifiers::NONE, Key::Enter) => {
                let last = self.get_last_line().to_string();
                if self.search_partial.is_some() {
                    self.exit_search_mode()
                };
                if self.command_history.len() >= self.history_size {
                    self.command_history.pop_front();
                }
                self.command_history.push_back(last.clone());

                self.force_cursor_to_end = true;
                self.history_cursor = None;
                self.truncate_scroll_back();
                (true, Some(last))
            }

            // in search mode the cursor is constrained to the inside of the
            // search prompt. In mormal mode the cursor is constrained to the
            // right of the prompt
            (Modifiers::NONE, Key::Delete) => {
                if let Some(search_partial) = &self.search_partial {
                    let last_off = self.last_line_offset();
                    if cursor > (last_off + SEARCH_PROMPT.len() - 2 + search_partial.len()) {
                        return (true, None);
                    }
                }
                (false, None)
            }
            (Modifiers::NONE, Key::ArrowRight) => {
                // nothing to do in normal mode. In search mode we need to
                // constrain the cursor to the search prompt
                if let Some(search_partial) = &self.search_partial {
                    let last_off = self.last_line_offset();

                    if cursor > (last_off + SEARCH_PROMPT.len() - 2 + search_partial.len()) {
                        return (true, None);
                    }
                }
                (false, None)
            }
            (Modifiers::NONE, Key::ArrowLeft) | (Modifiers::NONE, Key::Backspace) => {
                // in either mode dont allow motion (or deleting) into prompt

                let last_off = self.last_line_offset();
                match self.search_partial {
                    Some(_) => {
                        if cursor < (last_off + SEARCH_PROMPT_SLOT_OFF + 2) {
                            return (true, None);
                        }
                    }
                    None => {
                        if cursor < (last_off + self.prompt.len() + 1) {
                            return (true, None);
                        }
                    }
                }

                (false, None)
            }
            (Modifiers::NONE, Key::Escape) => {
                if self.search_partial.is_some() {
                    self.exit_search_mode()
                };
                self.history_cursor = None;
                (true, None)
            }

            // ctrl-r reverse search history
            (
                Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                    mac_cmd: false,
                    command: true,
                },
                Key::R,
            ) => {
                if self.search_partial.is_none() {
                    self.search_partial = Some(String::new());
                    self.enter_search_mode();
                } else {
                    self.history_back();
                }
                (true, None)
            }
            (Modifiers::NONE, Key::Tab) => {
                // off to tab completion land
                self.tab_complete();
                (true, None)
            }

            _ => (false, None),
        };

        return_value
    }

    fn history_back(&mut self) {
        let hc = match self.history_cursor {
            Some(hc) => hc,
            None => self.command_history.len(),
        };

        let mut hist_line = String::new();
        for i in (0..hc).rev() {
            match &self.search_partial {
                Some(search) => {
                    if search.is_empty() {
                        self.history_cursor = None;
                        break;
                    }

                    if self.command_history[i].contains(search) {
                        hist_line = self.command_history[i].clone();
                        self.history_cursor = Some(i);
                        break;
                    }
                }
                None => {
                    hist_line = self.command_history[i].clone();
                    self.history_cursor = Some(i);
                    break;
                }
            }
        }

        if !hist_line.is_empty() {
            let last = self.get_last_line();
            self.text = self.text.strip_suffix(last).unwrap_or("").to_string();
            self.text.push_str(&hist_line);
        }
    }

    fn last_line_offset(&self) -> usize {
        // offset in buffer of start of last line
        self.text.rfind('\n').map_or(0, |off| off + 1)
    }
    fn enter_search_mode(&mut self) {
        self.save_prompt = Some(self.prompt.clone());
        self.prompt = SEARCH_PROMPT.to_string();
        self.search_partial = Some(String::new());
        let last_off = self.last_line_offset();
        self.text.truncate(last_off);
        self.draw_prompt();
        self.force_cursor_to_end = true;
    }
    fn exit_search_mode(&mut self) {
        self.prompt = self.save_prompt.take().unwrap();

        let last_off = self.last_line_offset();
        self.text.truncate(last_off);
        self.draw_prompt();
        self.search_partial = None;
        self.force_cursor_to_end = true;
    }
    fn draw_prompt(&mut self) {
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            self.text.push('\n');
        }
        self.text.push_str(&self.prompt);
    }

    fn handle_kb(&mut self, ctx: &egui::Context) -> ConsoleEvent {
        // process all the key events in the queue
        // if they are meaningful to the console then use them and consume them
        // otherwise pass along to the textedit widget

        // current cursor position

        let cursor = if let Some(state) = egui::TextEdit::load_state(ctx, self.id) {
            if let Some(range) = state.cursor.char_range() {
                range.primary.index
            } else {
                0
            }
        } else {
            0
        };

        // a list of keys to consume

        let mut kill_list = vec![];
        let mut command = None;
        ctx.input(|input| {
            for event in &input.events {
                if let Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    modifiers,
                    repeat: _,
                } = event
                {
                    if *pressed {
                        let (kill, msg) = self.handle_key(key, *modifiers, cursor);
                        if kill {
                            kill_list.push((*modifiers, *key));
                        }
                        command = msg;
                        // if the user pressed enter we are done
                        if command.is_some() {
                            break;
                        }
                    }
                }
            }
        });

        // consume the keys we used
        for (modifiers, key) in kill_list {
            Self::consume_key(ctx, modifiers, key);
        }

        if let Some(command) = command {
            return ConsoleEvent::Command(command);
        }
        ConsoleEvent::None
    }
}
/// A builder for the console window
///
pub struct ConsoleBuilder {
    prompt: String,
    history_size: usize,
    scrollback_size: usize,
    tab_quote_character: char,
}

impl Default for ConsoleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleBuilder {
    /// Create a new console builder
    /// # Returns
    /// * `ConsoleBuilder` - the console builder
    ///
    pub fn new() -> Self {
        Self {
            prompt: ">> ".to_string(),
            history_size: 100,
            scrollback_size: 1000,
            tab_quote_character: '\'',
        }
    }
    /// Set the prompt for the console
    /// # Arguments
    /// * `prompt` - the prompt string
    ///
    /// # Returns
    /// * `ConsoleBuilder` - the console builder
    ///
    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }
    /// Set the history size for the console
    /// # Arguments
    /// * `size` - the size of the history
    ///
    /// # Returns
    /// * `ConsoleBuilder` - the console builder
    ///
    pub fn history_size(mut self, size: usize) -> Self {
        self.history_size = size;
        self
    }
    /// Set the scrollback size for the console
    /// # Arguments
    /// * `size` - the size of the scrollback
    ///
    /// # Returns
    /// * `ConsoleBuilder` - the console builder
    ///
    pub fn scrollback_size(mut self, size: usize) -> Self {
        self.scrollback_size = size;
        self
    }

    /// Set the character used to quote tab completed
    /// path containing spaces
    /// # Arguments
    /// * `quote` - character to use
    ///
    /// # Returns
    /// * `ConsoleBuilder` - the console builder
    ///
    pub fn tab_quote_character(mut self, quote: char) -> Self {
        self.tab_quote_character = quote;
        self
    }
    /// Build the console window
    /// # Returns
    /// * `ConsoleWindow` - the console window
    ///
    ///
    pub fn build(self) -> ConsoleWindow {
        let mut cons = ConsoleWindow::new(&self.prompt);
        cons.history_size = self.history_size;
        cons.scrollback_size = self.scrollback_size;
        cons.tab_quote = self.tab_quote_character;
        cons
    }
}
