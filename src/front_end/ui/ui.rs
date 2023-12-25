use std::{
    io,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::block::{Position, Title};
use slab_tree::*;
use crate::backend::tree::nodes::{LeafNodeType, CloneTree, NodeIndex};


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
    fn mark_item(&mut self) -> Option<String> {
        let item_pos = self.state.selected();
        match item_pos {
            Some(_) => {
                //if not multiselect remove clear selection
                if !self.multiselect {
                    for i in self.items.iter_mut() { i.1 = false; }
                }
                if let Some(pos) = item_pos {
                    //mark current item
                    self.items[pos].1 = !self.items[pos].1;
                    return None;
                }

                Some(String::from("invalid selection"))

            }
            None => { Some(String::from("please make a selection")) }
        }
    }
    fn get_selected(&self) -> Option<String> {
        for item in self.items.clone() {
            if item.1 {
                return Some(item.0)
            }
        }
        return None
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
    preview_tree: Tree<LeafNodeType>,
    vertical_index: usize,
    output: String,
}
impl App {
    fn new(tree: Tree<LeafNodeType>) -> App {
        let node_id = tree.root_id().expect("tree has no root");
        App {
            window: WindowType::App,
            question: String::new(),
            vertical_index: 0,
            preview_tree: tree.clone(),
            tree,
            node_id,
            input_mode: InputMode::Normal,
            cursor_position: 0,
            output: String::from("Press h for help"),
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
    fn next_item(&mut self, skip_child: bool) -> Option<Tree<String>> {
        fn check_recursively(app: &mut App, skip_child: bool, end_reached: &mut bool) {
            if let Some(node) = app.tree.get(app.node_id) {
                let first_child_opt = node.first_child();
                if !skip_child && first_child_opt.is_some() {
                    let child = first_child_opt.expect("Error, next child data could not be unwrap, despite it existing");
                    app.node_id = child.node_id();
                    match child.data() {
                        LeafNodeType::Text { name: _name } => {
                            check_recursively(app, false, end_reached);
                        }
                        _ => {}
                    }
                    app.set_question();
                } else if let Some(sibling) = node.next_sibling() {
                    app.node_id = sibling.node_id();
                    match sibling.data() {
                        LeafNodeType::Text { name: _name } => {
                            check_recursively(app, false, end_reached);
                        }
                        _ => {}
                    }
                    app.set_question();
                } else if let Some(parent) = node.parent() {
                    app.node_id = parent.node_id();
                    check_recursively(app, true, end_reached);
                } else {
                    app.vertical_index = app.tree.node_index(app.node_id);
                    *end_reached = true;

                }
                app.vertical_index = app.tree.node_index(app.node_id);
            }
        }
        let mut end_reached = false;
        check_recursively(self, skip_child, &mut end_reached);

        if end_reached {
            check_tree(&self.preview_tree)
        } else {
            None
        }

    }
    fn previous_item(&mut self) {
        if let Some(node) = self.tree.get(self.node_id) {
            if let Some(sibling) = node.prev_sibling() {
                self.node_id = sibling.node_id();
                self.vertical_index = self.tree.node_index(self.node_id);
                self.set_question();
            } else if let Some(parent) = node.parent() {
                self.node_id = parent.node_id();
                self.vertical_index = self.tree.node_index(self.node_id);
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
    fn update_preview_tree(&mut self) {
        fn walk_tree(node: NodeRef<LeafNodeType>, cur_index: &mut usize, tree_index: usize, node_id: &mut NodeId) {
            for child_ref in node.children() {
                *cur_index += 1;
                if *cur_index == tree_index {
                    *node_id = child_ref.node_id();
                    break
                } else if *cur_index > tree_index {
                    break
                } else {
                    walk_tree(child_ref, cur_index, tree_index, node_id);
                }
            }
        }
        let mut node_id = NodeId::from(self.preview_tree.root_id().expect("Error, Preview tree has no root"));
        walk_tree(self.preview_tree.root().expect("Error, tree has no root"), &mut 0, self.vertical_index, &mut node_id);
        if let Some(mut node) = self.preview_tree.get_mut(node_id) {
            match self.tree.get(self.node_id).expect("Error, no node to nodeID").data() {
                LeafNodeType::Text {name: _name} => {}
                LeafNodeType::TextInput {name: _name, input} => {
                    if !input.is_empty() {
                        *node.data() = LeafNodeType::Text {name: input.clone()}
                    }
                }
                LeafNodeType::Option {name: _name, options} => {
                    if let Some(item) = options.get_selected() {
                        *node.data() = LeafNodeType::Text{name: item}
                    }
                }
            }
        }

    }
}


pub fn init_ui(tree: Tree<LeafNodeType>) -> Result<Option<Tree<String>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new( tree);
    app.set_question();
    app.set_editing_mode();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    return res;
}

fn run_app<B: Backend>( terminal: &mut Terminal<B>, mut app: App, ) -> Result<Option<Tree<String>>, io::Error> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match &app.window {
                WindowType::App => {
                    match &app.input_mode {
                        InputMode::Normal if key.kind == KeyEventKind::Press => {
                            let mut node = app.tree.get_mut(app.node_id).expect("Unexpected Error, cannot get node from NodeID");
                            match node.data() {
                                LeafNodeType::Option { options, name: _name } => {
                                    match key.code {
                                        KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                                        KeyCode::Char('h') => { app.window = WindowType::Help }
                                        KeyCode::Char('s') => {
                                            app.update_preview_tree();
                                            match check_tree(&app.preview_tree){
                                                Some(tree) => {
                                                    return Ok(Some(tree))},
                                                None => { app.output = String::from("Cannot save, selection still missing"); }
                                            }
                                        }
                                        KeyCode::Down => options.next(),
                                        KeyCode::Up => options.previous(),
                                        KeyCode::Enter => {
                                            if let Some(err) = options.mark_item() {
                                                app.output = err;
                                            } else {
                                                app.update_preview_tree();
                                                match app.next_item(false) {
                                                    Some(tree) => {
                                                        println!("here");
                                                        return Ok(Some(tree)); },
                                                    None => {}
                                                }
                                                app.set_editing_mode();
                                            }
                                        },
                                        KeyCode::Right => {
                                            app.update_preview_tree();
                                            match app.next_item(false) {
                                                Some(tree) => return Ok(Some(tree)),
                                                None => {}
                                            }
                                        },
                                        KeyCode::Left => app.previous_item(),
                                        _ => {}
                                    }
                                }
                                LeafNodeType::TextInput { name: _name, input: _input } => match key.code {
                                    KeyCode::Char('i') | KeyCode::Char('e') | KeyCode::Down | KeyCode::Enter => {
                                        app.input_mode = InputMode::Editing;
                                        app.cursor_end();
                                    }
                                    KeyCode::Right => {
                                        app.update_preview_tree();
                                        match app.next_item(false) {
                                            Some(tree) => return Ok(Some(tree)),
                                            None => {}
                                        }
                                    },
                                    KeyCode::Left => app.previous_item(),
                                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                                    KeyCode::Char('h') => { app.window = WindowType::Help }
                                    KeyCode::Char('s') => {
                                        app.update_preview_tree();
                                        match check_tree(&app.preview_tree) {
                                            Some(tree) => return Ok(Some(tree)),
                                            None => {app.output = String::from("Cannot save, selection still missing");}
                                        }
                                    }
                                    _ => {}
                                }
                                _ => {
                                    match key.code {
                                        KeyCode::Char('h') => { app.window = WindowType::Help }
                                        KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                                        KeyCode::Char('s') => {
                                            match check_tree(&app.preview_tree) {
                                                Some(tree) => return Ok(Some(tree)),
                                                None => {app.output = String::from("Cannot save, selection still missing");}
                                            }
                                        }
                                        KeyCode::Left => app.previous_item(),
                                        _ => {}
                                    }
                                }
                            }
                        }
                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Char(to_insert) => {
                                if to_insert.is_ascii() {
                                    app.enter_char(to_insert);
                                } else {
                                    app.output = String::from("Non-ASCII char is not allowed");
                                }
                            }
                            KeyCode::Enter => {
                                app.input_mode = InputMode::Normal;
                                app.update_preview_tree();
                                match app.next_item(false) {
                                    Some(tree) => return Ok(Some(tree)),
                                    None => {}
                                }
                                app.set_editing_mode();
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
                            KeyCode::Char('q')  => return Ok(None),
                            _ => { app.window = WindowType::App }
                        }
                    }
                }
            }
        }
    }
}

fn check_tree(tree: &Tree<LeafNodeType>) -> Option<Tree<String>> {

    let root_string = tree.root().expect("Error tree has no root").data().get_name();

    let mut string_tree: Tree<String> = Tree::new();
    let root = string_tree.set_root(String::from(root_string));

    let mut partly_empty = false;
    fn walk_tree(node: NodeRef<LeafNodeType>, mut output_node: NodeMut<String>, partly_empty: &mut bool) {
        for child in node.children() {
            match child.data() {
                LeafNodeType::Text {name} => {
                    if name.is_empty() {
                        *partly_empty = true;
                    }
                    let string_node = output_node.append(String::from(name));
                    walk_tree(child, string_node, partly_empty);
                }
                _ => {
                    *partly_empty = true;
                }
            }
        }
    }
    walk_tree(tree.root().expect("Error, tree has no root"), string_tree.get_mut(root).unwrap(), &mut partly_empty);
    if partly_empty {
        None
    } else {
        Some(string_tree)
    }
}

fn ui(f: &mut Frame, app: &mut App) {

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Min(0),
        ])
        .split(f.size());

    f.render_widget(
        Block::new().borders(Borders::TOP)
            .title(
                Title::from(" Project Scaffold ").position(Position::Top).alignment(Alignment::Center),
            )
            .title(
                Title {content: Line::styled(" Press q to quit ", Style::default().fg(Color::LightYellow)), alignment: Some(Alignment::Left), position: Some(Position::Top) },
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
            let mut formatted_preview_tree = String::new();
            let _ = app.preview_tree.write_formatted(&mut formatted_preview_tree);
            let vec_of_preview_tree: Vec<Line> = formatted_preview_tree.lines().into_iter().enumerate().map(|(i,l)| {
                if i == app.vertical_index {
                    Line::from(l.bg(Color::LightMagenta))
                } else {
                    Line::from(l)
                }
            }).collect();

            f.render_widget(
                Paragraph::new(vec_of_preview_tree)
                    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)),
                inner_layout[1],
            );

            let input_indicator = match app.input_mode {
                InputMode::Editing => {
                    Paragraph::new("i:").style(Style::default().fg(Color::LightYellow).bold()).rapid_blink()
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
                .constraints([Constraint::Min(9), Constraint::Min(4), Constraint::Min(1)])
                .split(main_layout[1]);

            f.render_widget(
                Paragraph::new("  q, ESC: quit application \n  s: save and quit \n  ENT: Select Item in a list or enter text editing mode \n  i: Enter text editing mode \n  ←: Previous item \n  →: next item \n  ↑↓: go through a list")
                    .block(Block::default().borders(Borders::NONE).title("Normal Mode (v):".bold())),
                inner_layout[0],
            );

            f.render_widget(
                Paragraph::new("  ESC, ↑: Exit text editing \n  ENT: Submit Text")
                    .block(Block::default().borders(Borders::NONE).title("Editing Mode (i):".bold())),
                inner_layout[1],
            );

            f.render_widget(
                Paragraph::new("Note: Under Windows using \"~\" instead of \"C:\\Users\\username\" does currently not work".italic())
                    .block(Block::default().borders(Borders::NONE)),
                inner_layout[2],
            );
        }
    }
}


