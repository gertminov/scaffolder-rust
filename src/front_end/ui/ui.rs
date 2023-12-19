use std::{
    io,
};

use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::block::{Position, Title};
use slab_tree::*;
use crate::backend::tree::nodes::LeafNodeType;


enum InputMode {
    Normal,
    Editing,
}

pub struct StatefulList {
    state: ListState,
    items: Vec<(String, bool)>,
    multiselect: bool,
}
impl StatefulList {
    pub fn with_items(items: Vec<(String, bool)>, multiselect: bool) -> StatefulList {
        StatefulList {
            state: ListState::default().with_selected(Some(0)),
            items,
            multiselect,
        }
    }
    pub fn join_names_with(&self, separator: &str) -> String {
        let vec: Vec<String> = self.items.iter().map(|(name, _)| name.clone()).collect();
        return vec.join(separator);
    }
    pub fn clone(&self) -> StatefulList {
        return StatefulList { state: self.state.clone(), items: self.items.clone(), multiselect: self.multiselect.clone() };
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn mark_item(&mut self) -> String {
        let item_pos = self.state.selected();
        match item_pos {
            Some(_) => {
                //if not multiselect remove clear selection
                if !self.multiselect {
                    for i in self.items.iter_mut() { i.1 = false; }
                }
                //mark current item
                self.items[item_pos.unwrap()].1 = !self.items[item_pos.unwrap()].1;

                self.items[item_pos.unwrap()].0.clone()
            }
            None => { String::from("please make a selection") }//self.output = String::from("please make a selection")
        }
    }
}
enum WindowType {
    Help,
    App,
}
struct App {
    question: String,

    window: WindowType,
    input_mode: InputMode,
    cursor_position: usize,

    node_id: NodeId,
    tree: Tree<LeafNodeType>,
    formatted_tree: String,
    output: String,
}
impl App {
    fn new(formatted_tree: String, tree: Tree<LeafNodeType>) -> App {
        let node_id = tree.root_id().expect("tree has no root");
        App {
            window: WindowType::App,
            question: String::new(),
            formatted_tree,
            tree,
            node_id,
            input_mode: InputMode::Normal,
            cursor_position: 0,
            output: String::from("Press h for tooltips"),
        }
    }
    fn data_cloned(&self) -> Option<LeafNodeType> {
        if let Some(node) = self.tree.get(self.node_id) {
            return Some(node.data().clone());
        }
        return None;
    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }
    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }
    fn enter_char(&mut self, new_char: char) {
        let node_opt = self.tree.get_mut(self.node_id);
        match node_opt {
            Some(mut node) => {
                match node.data() {
                    LeafNodeType::TextInput { name: _name, input } => {
                        input.insert(self.cursor_position, new_char);
                        self.move_cursor_right();
                    }
                    _ => {}
                }
            }
            None => {}
        }
    }
    fn delete_char(&mut self) {
        if let Some(mut node) = self.tree.get_mut(self.node_id) {
            match node.data() {
                LeafNodeType::TextInput { name: _name, input } => {
                    let is_not_cursor_leftmost = self.cursor_position != 0;
                    if is_not_cursor_leftmost {
                        input.remove(self.cursor_position - 1);
                        self.move_cursor_left();
                    }
                }
                _ => {}
            }
        }
    }
    fn clamp_cursor(&mut self, new_cursor_pos: usize) -> usize {
        if let Some(data) = self.data_cloned() {
            match data {
                LeafNodeType::TextInput { name: _name, input } => { new_cursor_pos.clamp(0, input.len()) }
                _ => { 1 }
            }
        } else { 0 }
    }
    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
    fn cursor_end(&mut self) {
        if let Some(data) = self.data_cloned() {
            match data {
                LeafNodeType::TextInput { name: _name, input } => {
                    self.cursor_position = input.len();
                }
                _ => {}
            }
        }
    }
    fn next_item(&mut self, skip_child: bool) {
        if let Some(node) = self.tree.get(self.node_id) {
            if !skip_child {
                if let Some(child) = node.first_child() {
                    self.node_id = child.node_id();
                    match child.data() {
                        LeafNodeType::Text { name: _name } => {
                            self.next_item(false);
                        }
                        _ => {}
                    }
                    self.set_question();
                } else if let Some(sibling) = node.next_sibling() {
                    self.node_id = sibling.node_id();
                    match sibling.data() {
                        LeafNodeType::Text { name: _name } => {
                            self.next_item(false);
                        }
                        _ => {}
                    }
                    self.set_question();
                } else if let Some(parent) = node.parent() {
                    self.output = String::from(parent.data().get_name());
                    self.node_id = parent.node_id();
                    self.next_item(true);
                }
            } else {
                if let Some(sibling) = node.next_sibling() {
                    self.node_id = sibling.node_id();
                    match sibling.data() {
                        LeafNodeType::Text { name: _name } => {
                            self.next_item(false);
                        }
                        _ => {}
                    }
                    self.set_question();
                } else if let Some(parent) = node.parent() {
                    self.output = String::from(parent.data().get_name());
                    self.node_id = parent.node_id();
                    self.next_item(true);
                }
            }
        }
        //self.set_editing_mode();
    }
    fn previous_item(&mut self) {
        if let Some(node) = self.tree.get(self.node_id) {
            if let Some(sibling) = node.prev_sibling() {
                self.node_id = sibling.node_id();
                self.set_question();
            } else if let Some(parent) = node.parent() {
                self.node_id = parent.node_id();
                self.set_question();
            }
        }
    }
    fn set_question(&mut self) {
        if let Some(node) = self.tree.get(self.node_id) {
            match node.data() {
                LeafNodeType::TextInput { name, input: _input } => {
                    self.question = String::from("Type in a ") + name.as_str();
                }
                LeafNodeType::Option { name, options: _options } => {
                    self.question = String::from("Chose an option for ") + name.as_str()
                }
                _ => {}
            }
        }
    }
    fn set_editing_mode(&mut self) {
        if let Some(node) = self.tree.get(self.node_id) {
            match node.data() {
                LeafNodeType::TextInput {name: _name, input:_input} => {
                    self.input_mode = InputMode::Editing;
                    self.cursor_end();
                }
                _ => { self.input_mode = InputMode::Normal; }
            }
        }
    }
}


pub fn init_ui(tree: Tree<LeafNodeType>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut formatted_tree = String::new();
    let _ = tree.write_formatted(&mut formatted_tree);

    let mut app = App::new(formatted_tree, tree);
    app.set_question();
    //app.set_editing_mode();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match &app.window {
                WindowType::App => {
                    match &app.input_mode {
                        InputMode::Normal if key.kind == KeyEventKind::Press => {
                            let mut node = app.tree.get_mut(app.node_id).unwrap();
                            match node.data() {
                                LeafNodeType::Option { options, name: _name } => {
                                    match key.code {
                                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                        KeyCode::Char('h') => { app.window = WindowType::Help }
                                        KeyCode::Down => options.next(),
                                        KeyCode::Up => options.previous(),
                                        KeyCode::Enter => {
                                            options.mark_item();
                                            app.next_item(false);
                                        },
                                        KeyCode::Right => app.next_item(false),
                                        KeyCode::Left => app.previous_item(),
                                        _ => {}
                                    }
                                }
                                LeafNodeType::TextInput { name: _name, input: _input } => match key.code {
                                    KeyCode::Char('i') | KeyCode::Char('e') | KeyCode::Down | KeyCode::Enter => {
                                        app.input_mode = InputMode::Editing;
                                        app.cursor_end();
                                    }
                                    KeyCode::Right => app.next_item(false),
                                    KeyCode::Left => app.previous_item(),
                                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                    KeyCode::Char('h') => { app.window = WindowType::Help }
                                    _ => {}
                                }
                                _ => {
                                    match key.code {
                                        KeyCode::Char('h') => { app.window = WindowType::Help }
                                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                        KeyCode::Left => app.previous_item(),
                                        _ => {}
                                    }
                                }
                            }
                        }
                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            //KeyCode::Enter => app.submit_message(),
                            KeyCode::Char(to_insert) => {
                                if to_insert.is_ascii() {
                                    app.enter_char(to_insert);
                                } else {
                                    app.output = String::from("Non-ASCII char is not allowed");
                                }
                            }
                            KeyCode::Enter => {
                                app.next_item(false);
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Backspace => { app.delete_char(); }
                            KeyCode::Left => { app.move_cursor_left(); }
                            KeyCode::Right => { app.move_cursor_right(); }
                            KeyCode::Esc | KeyCode::Up => { app.input_mode = InputMode::Normal; }
                            KeyCode::End => { app.cursor_end(); }
                            KeyCode::Home => { app.reset_cursor(); }
                            _ => {}
                        }
                        _ => {}
                    }
                }
                WindowType::Help => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q')  => return Ok(()),
                            _ => { app.window = WindowType::App }
                        }
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    f.render_widget(
        Block::new().borders(Borders::TOP)
            .title(
                Title::from(" Project Scaffold ").position(Position::Top).alignment(Alignment::Center),
            )
            .title(
                Title {content: Line::styled(" Press q to quit ", Style::default().fg(Color::LightYellow)), alignment: Some(Alignment::Left), position: Some(Position::Top) },
                //Title::from(" Press q | ESC to quit ").position(Position::Top).alignment(Alignment::Left),
            ),
        main_layout[0],
    );
    match app.window {
        WindowType::App => {
            let inner_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_layout[1]);

            let bottom_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(3), Constraint::Min(1)])
                .margin(1)
                .split(main_layout[2]);


            let data = app.data_cloned().expect("No data available to display");
            match data {
                LeafNodeType::Option { name: _name, options } => {
                    // Iterate through all elements in the `items`
                    let items: Vec<ListItem> =
                        options.items
                            .iter()
                            .map(|i| {
                                let mut style = Style::default();
                                let mut content = String::from("[ ] ");
                                if i.1 == true {
                                    content = String::from("[x] ");
                                    style = Style::default().fg(Color::LightYellow);
                                }
                                content.push_str(i.0.as_str());

                                let lines = vec![
                                    Line::styled(content, style)
                                ];
                                ListItem::new(lines)
                            })
                            .collect();

                    // Create a List from all list items and highlight the currently selected one
                    let items = List::new(items)
                        .block(Block::default().borders(Borders::NONE).title(app.question.as_str().bold()))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().bg(Color::LightBlue).add_modifier(Modifier::BOLD))
                        .highlight_symbol("> ");

                    let mut state = options.state.clone();
                    // We can now render the item list
                    f.render_stateful_widget(items, inner_layout[0], &mut state);
                }

                LeafNodeType::TextInput { name: _name, input } => {
                    match &app.input_mode {
                        InputMode::Normal => {}
                        // Hide the cursor. `Frame` does this by default, so we don't need to do anything here

                        InputMode::Editing => {
                            // Make the cursor visible
                            f.set_cursor(
                                main_layout[1].x + app.cursor_position as u16 + 3,
                                main_layout[1].y + 2,
                            )
                        }
                    }
                    let text_input = String::from("> ") + input.as_str();
                    f.render_widget(
                        Paragraph::new(text_input).wrap(Wrap { trim: false })
                            .block(Block::default().borders(Borders::NONE).title(app.question.as_str().bold())),
                        inner_layout[0],
                    );
                }
                _ => {}
            }
            f.render_widget(
                Paragraph::new(app.formatted_tree.clone())
                    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)),
                inner_layout[1],
            );

            let input_indicator = match app.input_mode {
                InputMode::Editing => {
                    Paragraph::new("i:").style(Style::default().fg(Color::LightYellow).bold())
                }
                _ => {
                    Paragraph::new("v:").style(Style::default().fg(Color::LightBlue).bold())
                }
            };
            f.render_widget(
                input_indicator
                    .block(Block::default().borders(Borders::NONE)),
                bottom_layout[0],
            );

            f.render_widget(
                Paragraph::new(app.output.clone())
                    .block(Block::default().borders(Borders::NONE)),
                bottom_layout[1],
            );
        }
        WindowType::Help => {
            let inner_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(7), Constraint::Min(5)])
                .split(main_layout[1]);

            f.render_widget(
                Paragraph::new("  q, ESC: quit application \n  i, ENTER: Select Item in a list or enter text editing mode \n  ←: Previous item \n  →: next item \n  ↑↓: go through a list")
                    .block(Block::default().borders(Borders::NONE).title("Normal Mode (v):".bold())),
                inner_layout[0],
            );

            f.render_widget(
                Paragraph::new("  ESC, ↑: Exit text editing \n  ENTER: Submit Text")
                    .block(Block::default().borders(Borders::NONE).title("Editing Mode (i):".bold())),
                inner_layout[1],
            );
        }
    }


}


