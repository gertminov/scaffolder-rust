use std::{
    //error::Error,
    io,
    fmt,
};

use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::block::{Position, Title};

//use termtree::Tree;

enum InputMode {
    Normal,
    Editing,
}

enum Data<'a> {
    Select(StatefulList<'a>),
    TextInput(String),
}

struct StatefulList<'a> {
    state: ListState,
    items: Vec<(&'a str, bool)>,
    multiselect: bool,
}
impl<'a> StatefulList<'a>  {
    fn with_items(items: Vec<(&'a str, bool)>, multiselect: bool) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
            multiselect,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
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

    fn unselect(&mut self) {
        self.state.select(None);
    }
    fn mark_item(&mut self) -> &str {
        let item_pos = self.state.selected();
        match item_pos {
            Some(_) => {
                //if not multiselect remove clear selection
                if !self.multiselect {
                        for i in self.items.iter_mut() { i.1 = false; }
                    }
                //mark current item
                self.items[item_pos.unwrap()].1 = !self.items[item_pos.unwrap()].1;

                self.items[item_pos.unwrap()].0
            }
            None => {"please make a selection"}//self.output = String::from("please make a selection")
        }
    }
}

struct App<'a> {
    question: String,
    content: Data<'a>,
    tree_content: String,
    input: String,
    input_mode: InputMode,
    cursor_position: usize,
    output: String,
}

impl<'a> App<'a> {
    fn new(question: String, tree_content: String) -> App<'a> {
        App {
            question,
            content:Data::Select(StatefulList::with_items(vec![
                ("item0", false),
                ("item1", false),
                ("item2", false),
                ("item3", false),
                ("item4", false),
                ("item5", false),
                ], false)),
            tree_content,
            input: String::new(),
            input_mode: InputMode::Normal,
            cursor_position: 0,
            output: String::new(),
        }
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
        match &mut self.content {
            Data::TextInput(input) => {
                input.insert(self.cursor_position, new_char);
                self.move_cursor_right();
            }
            _=> {}
        }

    }
    fn delete_char(&mut self) {
        match &mut self.content {
            Data::TextInput(input) => {
                let is_not_cursor_leftmost = self.cursor_position != 0;
                if is_not_cursor_leftmost {
                    // Method "remove" is not used on the saved text for deleting the selected char.
                    // Reason: Using remove on String works on bytes instead of the chars.
                    // Using remove would require special care because of char boundaries.

                    let current_index = self.cursor_position;
                    let from_left_to_current_index = current_index - 1;

                    // Getting all characters before the selected character.
                    let before_char_to_delete = input.chars().take(from_left_to_current_index);
                    // Getting all characters after selected character.
                    let after_char_to_delete = input.chars().skip(current_index);

                    // Put all characters together except the selected one.
                    // By leaving the selected one out, it is forgotten and therefore deleted.
                    *input = before_char_to_delete.chain(after_char_to_delete).collect();
                    self.move_cursor_left();
                }
            }
            _ => {}
        }
    }
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        match &self.content {
            Data::TextInput(input) => { new_cursor_pos.clamp(0, input.len()) },

            _ => {1}
        }
    }
    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
    fn cursor_end(&mut self) {
        match &self.content {
            Data::TextInput(input) => {
                self.cursor_position = input.len();
            },

            _ => {}
        }

    }
    /*fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }*/

    fn next_content(&mut self) {
        self.content = Data::TextInput(String::from("test"))
    }
/*
    fn previous_item(&mut self) {
    }

    fn parent() {
    }

    fn next_parent() {
    }

    fn previous_parent() {
    }

*/

}


pub fn init_ui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let  app = App::new( String::from("Chose: "),String::from("Preview here"));

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
                match &app.input_mode {
                    InputMode::Normal if key.kind == KeyEventKind::Press =>
                        match &mut app.content {
                            Data::Select(list) => match key.code {
                                KeyCode::Char('q') | KeyCode::Esc=> return Ok(()),

                                KeyCode::Left => list.unselect(),
                                KeyCode::Down => list.next(),
                                KeyCode::Up =>   list.previous(),
                                KeyCode::Enter => {list.mark_item();},
                                KeyCode::Tab => {list.multiselect = !list.multiselect },
                                KeyCode::Right => app.next_content(),
                                _ => {}
                            }
                            Data::TextInput(_) => match key.code {
                                KeyCode::Char('e') | KeyCode::Down | KeyCode::Enter => {
                                    app.input_mode = InputMode::Editing;
                                    app.cursor_end();
                                }
                                KeyCode::Right => app.next_content(),

                                KeyCode::Char('q') | KeyCode::Esc=> return Ok(()),
                                _=> {}
                            }
                        }

                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        //KeyCode::Enter => app.submit_message(),
                        KeyCode::Char(to_insert) => {
                            if to_insert.is_ascii() {
                                app.enter_char(to_insert);
                            }
                            else {
                                app.output = String::from("Non-ASCII char is not allowed");
                            }
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
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.size());

    let padding = Padding::uniform(10);
    f.render_widget(
        Block::new().borders(Borders::TOP)
            .title(
                Title::from(" Project Scaffolder ").position(Position::Top).alignment(Alignment::Center),
            )
            .title(
                Title::from(" Press q to quit").position(Position::Top).alignment(Alignment::Right),
            )
            .style(Style::default()),
        main_layout[0],
    );

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[1]);

    match &app.content {
        Data::Select(list) => {

            // Iterate through all elements in the `items`
            let items: Vec<ListItem> =
                list.items
                    .iter()
                    .map(|i| {
                        let mut style = Style::default();
                        let mut content = String::from("[ ] ");
                        if i.1 == true {
                            content = String::from("[x] ");
                            style = Style::default().fg(Color::Yellow);
                        }
                        content.push_str(i.0);

                        let lines = vec![
                            Line::styled(content, style)
                        ];
                        ListItem::new(lines)
                    })
                    .collect();

            // Create a List from all list items and highlight the currently selected one
            let items = List::new(items)
                .block(Block::default().borders(Borders::NONE).title(app.question.clone()))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().bg(Color::LightBlue).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            let mut state = list.state.clone();
            // We can now render the item list
            f.render_stateful_widget(items, inner_layout[0], &mut state);
        }

        Data::TextInput(content) => {

            match &app.input_mode {
                InputMode::Normal => {}
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here


                InputMode::Editing => {
                    // Make the cursor visible
                    f.set_cursor(
                        main_layout[1].x + app.cursor_position as u16  + 3,
                        main_layout[1].y + 2,
                    )
                }
            }
            let text_input = String::from("> ") + content;
            f.render_widget(
                Paragraph::new(text_input).wrap(Wrap {trim: false })
                    .block(Block::default().borders(Borders::NONE).title(app.question.clone())),
                inner_layout[0],
            );

        }
    }
    f.render_widget(
        Paragraph::new(app.tree_content.clone())
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)),
        inner_layout[1],
    );


    f.render_widget(
        Paragraph::new(app.output.clone())
            .block(Block::default().borders(Borders::NONE)),
        main_layout[2],
    );
}




/*
fn ui(frame: &mut Frame) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.size());

    frame.render_widget(
        Block::new().borders(Borders::ALL).title("Title Bar"),
        main_layout[0],
    );

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)])
        .split(main_layout[1]);

    let left_block = Block::default().borders(Borders::NONE);
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Preview"),
        inner_layout[1],
    );

    let items = [ListItem::new("Item 1"), ListItem::new("Item 2"), ListItem::new("Item 3")];
    let list =List::new(items)
        .block(left_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
    frame.render_widget(
        list,
        inner_layout[0],

    );
}
*/
