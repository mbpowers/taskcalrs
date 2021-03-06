use std::io::{stdin, stdout, Write, Error, BufReader, BufRead};
use std::{thread, time::Duration};
use std::str::FromStr;
use std::fs::File;
use std::fmt::Display;
use std::process::Command;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use tui::Terminal;
use tui::text::{ Spans, Span };
use tui::style::{ Color, Style, Modifier};
use tui::backend::TermionBackend;
use tui::widgets::*;
use tui::layout::{Layout, Rect, Constraint, Direction, Alignment, Margin};
use chrono::{Datelike, Local, NaiveDate, NaiveTime, NaiveDateTime};
use regex::Regex;
// use num_traits::cast::FromPrimitive;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

// impl Display for Mode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match &self {
//             Mode::Tasks => write!(f, "Contexts"),
//             Mode::Schedule => write!(f, "Schedule"),
//             Mode::Calendar => write!(f, "Calendar"),
//         }
//     }
// }

struct ModeSelection { mode: String, leader: Key }
impl ModeSelection {
    fn new(mode: String, leader: Key) -> ModeSelection{ ModeSelection{ mode, leader } }
    fn next(&mut self) { match &*self.mode {
            "Calendar" => self.mode = "Contexts".to_string(),
            "Contexts" => self.mode = "Schedule".to_string(),
            "Schedule" => self.mode = "Calendar".to_string(),
            &_ => (),
    } }
    fn prev(&mut self) { match &*self.mode {
            "Contexts" => self.mode = "Calendar".to_string(),
            "Schedule" => self.mode = "Contexts".to_string(),
            "Calendar" => self.mode = "Schedule".to_string(),
            &_ => (),
    } }
    fn calendar(&mut self) { self.mode = "Calendar".to_string() }
    fn contexts(&mut self) { self.mode = "Contexts".to_string() }
    fn schedule(&mut self) { self.mode = "Schedule".to_string() }
    fn reset_leader(&mut self) { self.leader = Key::Null }
}

// struct SchedSelection { items: Vec<String>, state: ListState }
// impl SchedSelection {
//     fn new(items: Vec<String>) -> SchedSelection { SchedSelection { items, state: ListState::default(), } }
//     fn set_items(&mut self, items: Vec<String>) { self.items = items; self.state = ListState::default(); }
//     fn unselect(&mut self) { self.state.select(None); }
//     fn select(&mut self, i: usize) { self.state.select(Some(i)); }
//     fn next(&mut self) {
//         let i = match self.state.selected() {
//             Some(i) => { if i >= self.items.len() - 1 { 0 } else { i + 1 } }
//             None => 0,
//         };
//         self.state.select(Some(i));
//     }
//     fn prev(&mut self) {
//         let i = match self.state.selected() {
//             Some(i) => { if i == 0 { self.items.len() - 1 } else { i - 1 } }
//             None => 0,
//         };
//         self.state.select(Some(i));
//     }
// }

struct DateSelection { date: NaiveDate, time: NaiveTime, event: Option<Event> }
// impl Display for DateSelection { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.date.to_string()) } }
impl DateSelection {
    fn date(date: NaiveDate) -> DateSelection { DateSelection { date, time: NaiveTime::from_hms(0,0,0), event: None} }
    fn datetime(date: NaiveDate, time: NaiveTime) -> DateSelection { DateSelection { date, time, event: None } }
    fn clone(&self) -> DateSelection { DateSelection { date: self.date, time: self.time, event: None } }
    fn set_date(&mut self, date: NaiveDate) { self.date = date }
    fn set_time(&mut self, time: NaiveTime) { self.time = time }
    // fn set_event(&mut self, string: String) { self.event = Some(Event::from_str(string)) }
    fn set_event(&mut self, event: Option<Event>) { self.event = event }
    fn event(&self) -> Option<Event> { self.event.clone() }

    fn prev_year(&mut self) {
        self.date = if self.is_leap_day() {
            NaiveDate::from_ymd(self.year()-1, self.month(), 28)
        } else { self.date.with_year(self.year()-1).unwrap() }
    }
    fn next_year(&mut self) {
        self.date = if self.is_leap_day() {
            NaiveDate::from_ymd(self.year()+1, self.month(), 28)
        } else { self.date.with_year(self.year()+1).unwrap() }
    }
    fn prev_month(&mut self) {
        let day = self.day(); let mut month = self.month(); let mut year = self.year();
        if month == 1 { year -= 1; month = 12 } else { month -= 1 }
        self.date = NaiveDate::from_ymd(year, month, 1);
        self.date = if day > self.month_length() { self.date.with_day(self.month_length()).unwrap() } else { self.date.with_day(day).unwrap() }
    }
    fn next_month(&mut self) {
        let day = self.day(); let mut month = self.month(); let mut year = self.year();
        if month == 12 { year += 1; month = 1 } else { month += 1 }
        self.date = NaiveDate::from_ymd(year, month, 1);
        self.date = if day > self.month_length() { self.date.with_day(self.month_length()).unwrap() } else { self.date.with_day(day).unwrap() }
    }
    fn prev_week(&mut self) { self.date = self.date.pred().pred().pred().pred().pred().pred().pred() }
    fn next_week(&mut self) { self.date = self.date.succ().succ().succ().succ().succ().succ().succ() }
    fn prev_day(&mut self) { self.date = self.date.pred() }
    fn next_day(&mut self) { self.date = self.date.succ() }
    fn prev_hour(&mut self) {
        if self.hour() > 0 { self.time = NaiveTime::from_hms(self.hour()-1, self.minute(), self.second()) }
        else { self.time = NaiveTime::from_hms(23, self.minute(), self.second()); self.prev_day() }}
    fn next_hour(&mut self) {
        if self.hour() < 23 { self.time = NaiveTime::from_hms(self.hour()+1, self.minute(), self.second()) }
        else { self.time = NaiveTime::from_hms(0, self.minute(), self.second()); self.next_day() }}

    fn second(&self) -> u32 { return self.time.format("%S").to_string().parse::<u32>().unwrap() }
    fn minute(&self) -> u32 { return self.time.format("%M").to_string().parse::<u32>().unwrap() }
    fn hour(&self) -> u32 { return self.time.format("%H").to_string().parse::<u32>().unwrap() }
    fn day(&self) -> u32 { return self.date.day() }
    fn month(&self) -> u32 { return self.date.month() }
    fn year(&self) -> i32 { return self.date.year() }

    fn first(&self) -> NaiveDate { return NaiveDate::from_ymd(self.year(), self.month(), 1) }
    // fn weeks(&self) -> u32 { return week_diff(self.date) }
    fn month_length(&self) -> u32 {
        let year = self.year();
        match self.month() {
            1 | 3 | 5 | 7 | 8 | 10 | 12 =>  return 31,
            4 | 6 | 9 | 11 => return 30,
            2 => if year % 4 == 0 && ( year % 400 == 0 || year % 100 != 0 ) { return 29 } else { return 28 }
            _ => 0
        }
    }
    fn month_string(&self) -> String {
        return match self.month() {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "",
        }.to_string()
    }
    fn is_leap_year(&self) -> bool {
        let year = self.year();
        year % 4 == 0 && (year % 400 == 0 || year % 100 != 0)
    }
    fn is_leap_day(&self) -> bool { self.is_leap_year() && self.day() == 29 && self.month() == 2 }

    // fn select(&mut self) { self.selected = true }
    // fn deselect(&mut self) { self.selected = false }
    // fn toggle(&mut self) { self.selected = !self.selected }

    // fn index_of(&self) -> u8 { month_in_days(pmonth(self.date))-7-self.first().weekday().num_days_from_sunday() as u8 + self.day() }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Cycle { Never, Daily, Weekly, Monthly, Yearly }
impl FromStr for Cycle {
    type Err = ();
    fn from_str(input: &str) -> Result<Cycle, Self::Err> {
        match input {
            "N" => Ok(Cycle::Never),
            "D"  => Ok(Cycle::Daily),
            "W"  => Ok(Cycle::Weekly),
            "M"  => Ok(Cycle::Monthly),
            "Y" => Ok(Cycle::Yearly),
            _      => Err(()),
        }
    }
}
impl Cycle {  }

#[derive(Debug, PartialEq, Clone)]
struct Event { name: String, time: NaiveTime, date: NaiveDate, duration: u8, repeat_cycle: Cycle, repeat_occurences: u8, color: Color, task_modifier: String}
impl Event {
    fn new(name: String, time: NaiveTime, date: NaiveDate, duration: u8, repeat_cycle: Cycle, repeat_occurences: u8, color: Color, task_modifier: String) -> Event {
        Event{name, time, date, duration, repeat_cycle, repeat_occurences, color, task_modifier}
    }
    fn from_str(str: String) -> Event {
        let substr: Vec<String> = str.split_whitespace().map(|s| s.to_string()).collect();
        let color = match &*substr[0] {
            "Red" => Color::Red, "LightRed" => Color::LightRed, "Yellow" => Color::Yellow, "LightYellow" => Color::LightYellow,
            "Green" => Color::Green, "LightGreen" => Color::LightGreen, "Blue" => Color::Blue, "LightBlue" => Color::LightBlue,
            "Cyan" => Color::Cyan, "LightCyan" => Color::LightCyan, "Magenta" => Color::Magenta, "LightMagenta" => Color::LightMagenta,
            "Black" => Color::Black, "DarkGray" => Color::DarkGray, "Gray" => Color::Gray, "White" => Color::White,
            &_ => Color::Reset,
        };
        let date = NaiveDate::from_ymd(substr[1].parse::<i32>().unwrap(), substr[2].parse::<u32>().unwrap(), substr[3].parse::<u32>().unwrap());
        let time = NaiveTime::from_hms(substr[4].parse::<u32>().unwrap(), 0, 0);
        let duration = substr[5].parse::<u8>().unwrap();
        let repeat_cycle = Cycle::from_str(&substr[6]).unwrap();
        let repeat_occurences = substr[7].as_bytes().iter().fold(0, |acc, &b| acc*2 + b - 48 as u8);
        // println!("{}", format!("{:08b}", repeat_occurences));
        let name = &substr[8];
        let task_modifier = substr[9].to_string();
        Event{name: name.to_string(), time, date, duration, repeat_cycle, repeat_occurences, color, task_modifier}
    }
    fn clone(&self) -> Event {
        Event{ name: self.name.clone(), time: self.time, date: self.date, duration: self.duration, repeat_cycle: self.repeat_cycle,
            repeat_occurences: self.repeat_occurences, color: self.color, task_modifier: self.task_modifier.clone() }
    }
}


struct TaskSelection { items: Vec<Task>, state: ListState, project: String, tags: Vec<String> }
impl TaskSelection {
    fn from_tasks(items: Vec<Task>) -> TaskSelection {
        TaskSelection { items, state: ListState::default(), project: "".to_string(), tags: vec!("".to_string()) }
    }
    fn from_type(items: Vec<Task>, project: String, tags: Vec<String>) -> TaskSelection {
        TaskSelection { items, state: ListState::default(), project, tags }
    }
    fn new(items: Vec<Task>, project: String, tags: Vec<String>) -> TaskSelection {
        TaskSelection { items, state: ListState::default(), project, tags }
    }
    fn set_items(&mut self, items: Vec<Task>) { self.items = items; self.state = ListState::default(); }
    fn unselect(&mut self) { self.state.select(None); }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => { if i >= self.items.len() - 1 { 0 } else { i + 1 } }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => { if i == 0 { self.items.len() - 1 } else { i - 1 } }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn populate(&mut self, tasks: Vec<Task>) {
        for task in tasks {
            let mut tag_match = false;
            let mut project_match = false;
            for tag in &self.tags {
                if !task.tags.contains(&tag) { tag_match = true }
                else if tag == "" { tag_match = true }
            }
            if self.project == "" || self.project == task.project { project_match = true }
            if project_match && tag_match {
                self.items.push(task)
            }
        }
    }

}

#[derive(Debug, Clone)]
struct Task { uuid: String, id: u32, deps: Vec<String>, project: String, tags: Vec<String>, description: String, annotation: Vec<String>, urg: f32, status: String }
// impl Display for Task { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     // write!(f, "{}, {:?}, {}, {:?}, {}, {}, {}", self.id, self.deps, self.project, self.tags, self.description, self.urg, self.status)
//     // write!(f, "{: >2}  ", self.id)?;
//     // if self.project!="" {write!(f, "{}  ", self.project)?;}
//     // if self.tags[0]!="" {write!(f, "{:?}  ", self.tags)?;}
//     write!(f, "{}  {}", self.description, self.urg)
// } }
impl Task {
    fn specifier(&self) -> String { "uuid:".to_owned()+&self.uuid }
    fn complete(&mut self) {
        println!("COMPLETING");
        let output = Command::new("task").args([&self.specifier(), "done"]).output().unwrap();
        if output.status.success() { self.status = "completed".to_string(); }
    }
    fn reopen(&mut self) {
        println!("REOPENING");
        let output = Command::new("task").args([&self.specifier(), "mod", "status:pending", "end:"]).output().unwrap();
        if output.status.success() { self.status = "pending".to_string(); }
    }
    fn toggle_complete(&mut self) {
        if self.status == "pending" { self.complete() } else if self.status == "completed" { self.reopen() }
    }
}

#[derive(Clone, Debug)]
struct Context {name: String, project: String, tags: Vec<String>, tasks: Vec<Task>, state: ListState, hidden: usize}
impl Context {
    fn new(name: String, project: String, tags: Vec<String>) -> Context { Context {name, project, tags, tasks: Vec::<Task>::new(), state: ListState::default(), hidden: 0 } }
    fn general() -> Context { Context { name: "General".to_string(), project: "none".to_string(), tags: vec!("none".to_string()), tasks: Vec::<Task>::new(), state: ListState::default(), hidden: 0 } }
    fn from_file(string: String) -> Context {
        let substrings: Vec<String> = string.split(' ').map(|s| s.to_string()).collect();
        // for sub in &substrings {
            // println!("{}", sub);
        // }
        Context { name: substrings[0].clone(), project: substrings[1].clone(), tags: substrings[2..].to_vec(), tasks: Vec::<Task>::new(), state: ListState::default(), hidden: 0  }
    }
    fn from_event(event: Event) -> Context {
        let substrings: Vec<String> = event.task_modifier.split(':').map(|s| s.to_string()).collect();
        Context { name: event.name, project: substrings[0].clone(), tags: substrings[1..].to_vec(), tasks: Vec::<Task>::new(), state: ListState::default(), hidden: 0  }
    }
    fn populate(&mut self, tasks: Vec<Task>) {
        for task in tasks {
            let mut tag_match = true;
            let mut project_match = false;
            for tag in &self.tags {
                if !task.tags.contains(&tag) { tag_match = false }
                if tag == "none" { tag_match = true }
            }
            if self.project == "none" || self.project == "" || self.project == task.project { project_match = true }
            if project_match && tag_match { self.tasks.push(task) }
        }
    }
    fn deselect(&mut self) { self.state.select(None); }
    fn selected(&mut self) -> Option<&mut Task> {
        match self.state.selected() {
            Some(i) => Some(&mut self.tasks[i]),
            None => None,
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => { if i >= self.tasks.len() - self.hidden - 1 { 0 } else { i + 1 } }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => { if i == 0 { self.tasks.len() - self.hidden - 1 } else { i - 1 } }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
#[derive(Clone)]
enum Annotations { All, Selected }

#[derive(Clone)]
struct ContextSelection {index: usize, selected: bool, show_annos: bool, all_annos: bool, contexts: Vec<Context>}
impl ContextSelection {
    fn new() -> ContextSelection { ContextSelection{ index: 0, selected: true, show_annos: false, all_annos: false, contexts: Vec::<Context>::new()} }

    fn next(&mut self) { if self.index < self.len() - 1{self.index += 1} else {self.index = 0}}
    fn prev(&mut self) { if self.index > 0 {self.index -= 1} else {self.index = self.len() -1}}

    fn context(&mut self) -> &mut Context { &mut self.contexts[self.index] }
    fn next_task(&mut self) { self.context().next() }
    fn prev_task(&mut self) { self.context().prev() }
    fn task(&mut self) -> Option<&mut Task> { self.context().selected() }
    // fn next_task(&mut self) { self.contexts[self.index].next() }
    // fn prev_task(&mut self) { self.contexts[self.index].prev() }

    fn is_sel(&self) -> bool { self.selected.clone() }
    fn select(&mut self) { self.selected = true }
    fn deselect(&mut self) { self.selected = false }
    fn toggle_selected(&mut self) { self.selected = !self.selected }
    fn toggle_show_annos(&mut self) { self.show_annos = !self.show_annos }
    fn toggle_all_annos(&mut self) { self.all_annos = !self.all_annos }

    fn push(&mut self, context: Context) { self.contexts.push(context) }
    fn set_sched_context(&mut self, context: &Context) { self.contexts.push(context.clone()) }
    // fn context(&self) -> &Context { &self.contexts()[self.index as usize] }
    fn len(&self) -> usize { self.contexts.len() }
    fn from_cli() -> Vec<Task> {
        // json: description due end entry imask:0 modified parent priority recur rtype status until uuid wait urgency
        let output = Command::new("task").arg("export").output().unwrap();
        let mut tasks: Vec<Task> = Vec::new();
        if output.status.success() {
            let buf = String::from_utf8(output.stdout).unwrap().to_string();
            let lines: Vec<String> = buf.split('\n').map(|s| s.to_string()).collect();
            for line in lines.iter() {
                if line.len() < 2 { continue }
                let (mut uuid, mut id, mut deps, mut project, mut tags, mut description, mut annotation, mut urg, mut status)
                    = ("".to_string(), 0, Vec::<String>::new(), "".to_string(), Vec::<String>::new(), "".to_string(), Vec::<String>::new(), -1.0, "".to_string());
                let re = Regex::new(r##"[^\{\},\[]*\[[^\]]*]|[^\{\},]+"##).unwrap();
                let re_annotations = Regex::new(r"\\(.)").unwrap();
                // let re = Regex::new(r"(?P<first>\w+)\s+(?P<second>\w+)").unwrap();
                // let result = re.replace("deep fried", "${first}_$second");
                let substrings = re.find_iter(line).map(|m| m.as_str());
                for sub in substrings {
                    let v = sub.split_once(':').unwrap();
                    let (key, value) = (v.0.replace('\"', "").to_string(), v.1.replace(&['\"','\n'][..], ""));
                    match &*key {
                        "uuid" => uuid = value,
                        "id" => id = value.parse::<u32>().unwrap(),
                        "depends" => deps = value.replace(&['[',']'][..], "").split(",").map(|s| s.to_string()).collect(),
                        "project" => project = value,
                        "tags" => tags = value.replace(&['[',']'][..], "").split(",").map(|s| s.to_string()).collect(),
                        "description" => description = value,
                        "annotations" => {
                            let mut out = Vec::<String>::new();
                            let vec = value.replace(&['[',']','{','}'][..], "").split(",").map(|s| s.split_once(':').unwrap().1.to_string()).collect::<Vec<String>>();
                            for i in (0..vec.len()).step_by(2) {
                                let mut date = vec[i].split('T').map(|s| s.to_string()).collect::<Vec<String>>()[0].clone();
                                date.insert(4, '-'); date.insert(7, '-');
                                let text = re_annotations.replace_all(&vec[i+1], "${1}");
                                out.push("    ".to_string()+&date +" "+&text);
                            }
                            annotation = out;
                        }
                        "status" => status = value,
                        "urgency" => urg = value.parse::<f32>().unwrap(),
                        &_ => (),
                    }
                }
                tasks.push(Task {uuid, id, deps, project, tags, description, annotation, urg, status});
            }
        }
        tasks
    }
}




fn main() -> Result<(), Error> {
    // let stdin = stdin();
    let stdin = termion::async_stdin();
    let mut stdout = stdout().into_raw_mode()?;
    write!(
        stdout,
        "{}{}",
        termion::cursor::Goto(1, 1),
        termion::clear::All
    )
   .unwrap();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    println!("");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");
    // println!("test");

    // Instantiate Variables
    let mut show_keys = false;
    let mut show_completed = true;
    let mut show_title = true;
    let mut today = Local::today().naive_local();
    let mut time = Local::now().time();
    let mut date_selection = DateSelection::datetime(today, time);
    let mut mode_selection = ModeSelection::new("Calendar".to_string(), Key::Null);
    let mut sched_list: Vec<Event> = Vec::new();
    let file = File::open("schedule")?;
    for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
        if !line.starts_with("#") && line.len() > 0 { sched_list.push(Event::from_str(line)); }
    }

    let task_list: Vec<Task> = ContextSelection::from_cli();
    let mut context_selection = ContextSelection::new();
    let mut general_context = Context::general();
    general_context.populate(task_list.clone());
    context_selection.push(general_context);

    let file = File::open("contexts")?;
    for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
        let mut context = Context::from_file(line.clone());
        context.populate(task_list.clone());
        if !line.starts_with("#") && line.len() > 0 { context_selection.push(context); }
    }



    // Main Loop
    let mut it = stdin.keys();
    loop {
        today = Local::today().naive_local();
        time = Local::now().time();
        terminal.draw(|f| {
            if show_title {
                let (foreground, background) = create_title(f.size());
                f.render_widget(background, f.size());
                let rect = Rect { x: (f.size().width-78)/2, y: (f.size().height-9)/2, width: 78, height: 9 };
                f.render_widget(foreground, rect);
            } else {

                let rects = create_rects(f.size(), show_keys);

                // Calendar
                let cal_title = date_selection.month_string()+" "+&date_selection.day().to_string()+" "+&date_selection.date.year().to_string();
                let calendar = Block::default()
                    .title(Spans::from(vec![ Span::styled(cal_title, Style::default().fg(Color::Blue)), ]))
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg( if mode_selection.mode == "Calendar".to_string() {Color::Red} else {Color::White}))
                    .borders(Borders::ALL);
                let date_nums = create_dates(&date_selection);
                let mut dates = Vec::new();
                let cal_rects = create_cal_rects(calendar.inner(rects[1]), show_keys);
                for (i, (rect, date)) in cal_rects.clone().into_iter().zip(date_nums.iter()).enumerate() {
                    dates.push((
                        Block::default()
                            .title(
                                Span::styled(
                                    if i < 7 { date.weekday().to_string() }
                                    else { format!("{: ^4}", format!("{: >2}", date.day().to_string())) },
                                    Style::default().fg(
                                        if i < 7 { Color::Blue }
                                        else if date.month() == date_selection.month().into() { Color::White }
                                        else { Color::Red }
                                    ).bg(
                                        if date == &today { Color::Red }
                                        else if date == &date_selection.date { Color::Yellow }
                                        else { Color::Reset }
                                    )
                                )
                            ),
                    rect));
                }
                f.render_widget(calendar, rects[1]);
                for date in dates { f.render_widget(date.0, date.1); }
                if show_keys {
                    f.render_widget(
                        Paragraph::new("H-L:yr J-K:mn h-l:wk j-k:dy")
                        .block(Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default().fg( if mode_selection.mode == "Calendar".to_string() {Color::Red} else {Color::White}))
                    ), cal_rects[cal_rects.len()-1]); }


                // Schedule
                // Outer Schedule Block
                let schedule = Block::default()
                    .title(Spans::from(vec![ Span::styled(format!("{}", time.format("%H:%M:%S")), Style::default().fg(Color::Blue)), ]))
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg( if mode_selection.mode == "Schedule".to_string() {Color::Red} else {Color::White}))
                    .borders(Borders::ALL);
                // let colors = [Color::Red, Color::Yellow, Color::Green, Color::Cyan, Color::LightBlue, Color::Blue, Color::Magenta];

                // Inner Schedule Block
                let sched_rects = create_sched_rects(schedule.inner(rects[2]), show_keys);
                let left_rects = sched_rects.0;
                let right_rects = sched_rects.1;
                for (i, rect) in left_rects.iter().enumerate() {
                    if show_keys && i == left_rects.len()-1 {
                        f.render_widget(
                            Paragraph::new("H-L:mn J-K:wk h-l:dy j-k:hr")
                            .block(Block::default()
                                .borders(Borders::TOP)
                                .border_style(Style::default().fg( if mode_selection.mode == "Schedule".to_string() {Color::Red} else {Color::White}))
                        ), left_rects[left_rects.len()-1]);
                    } else {
                        let hour = (date_selection.hour()+i as u32)%24;
                        // println!("{}, {}", time.format("%H").to_string(), format!("{:0>2}", hour.to_string()));
                        let mut style = Style::default() .bg(
                                            if time.format("%H").to_string() == format!("{:0>2}", hour.to_string()) {
                                                if date_selection.date == today.pred() && (date_selection.hour()+i as u32) >= 24 {Color::Red}
                                                else if date_selection.date == today && (date_selection.hour()+i as u32) < 24 {Color::Red}
                                                else {Color::Reset}
                                            }
                                            else {Color::Reset});
                        // if hour % 2 == 0 { style = style.add_modifier(Modifier::BOLD); }
                        let widget = Paragraph::new(format!("{: >2}", hour.to_string())).block(Block::default().style(style));
                        f.render_widget(widget, *rect);
                    }
                }
                // let colors = [Color::Red, Color::Yellow, Color::Green, Color::Cyan, Color::LightBlue, Color::Blue, Color::Magenta];
                let mut event_is_set = false;
                for event in sched_list.iter() {
                    let mut datetime = date_selection.clone();
                    let begin_time = event.time.format("%H").to_string().parse::<u32>().unwrap();
                    let end_time = (begin_time + event.duration as u32) % 24;
                    for (i, rect) in right_rects.iter().enumerate() {
                        let mut render = false;
                        match event.repeat_cycle {
                            Cycle::Never => { if begin_time <= datetime.hour() && datetime.hour() < end_time && datetime.date == event.date { render = true; } }
                            Cycle::Daily => {
                                if begin_time < end_time {
                                    if begin_time <= datetime.hour() && datetime.hour() < end_time { render = true; }
                                } else {
                                    if begin_time <= datetime.hour() && datetime.hour() > end_time { render = true; }
                                    else if begin_time >= datetime.hour() && datetime.hour() < end_time { render = true; }
                                }
                            }
                            Cycle::Weekly => {
                                // if ("1".to_string() + &"0".repeat((6-date_selection.date.weekday().num_days_from_sunday()) as usize)).as_bytes().iter().fold(0, |acc, &b| acc*2 + b - 48 as u8) & event.repeat_occurences != 0 {
                                if 2_u8.pow(6-date_selection.date.weekday().num_days_from_sunday()) & event.repeat_occurences != 0 {
                                    if begin_time <= datetime.hour() && datetime.hour() < end_time { render = true; }
                                }
                            }
                            _ => (),
                        }
                        if render {
                            if i==0 {
                                event_is_set = true;
                                if date_selection.event() == None || date_selection.event().unwrap() != event.to_owned() {
                                    date_selection.set_event(Some(event.clone()));
                                    let mut context = Context::from_event(event.clone());
                                    context.populate(task_list.clone());
                                    context_selection.contexts[0] = context;
                                }
                            }
                            let widget = Paragraph::new(if datetime.hour() == begin_time
                                                        || ( begin_time < end_time && begin_time <= date_selection.hour() && date_selection.hour() < end_time && datetime.hour() == date_selection.hour())
                                                        || ( begin_time > end_time
                                                             && ((begin_time <= datetime.hour() && datetime.hour() > end_time && datetime.hour() == date_selection.hour())
                                                                 || (begin_time >= datetime.hour() && datetime.hour() < end_time && datetime.hour() == date_selection.hour())
                                                                 ))
                                                        { format!("{: ^width$}", event.name.clone(), width=rect.width as usize) } else { "".to_string() })
                                .block(Block::default()
                                    .style(Style::default()
                                        .bg(event.color)
                                        .fg(if event.color == Color::White { Color::Black } else { Color::White })));
                            f.render_widget(widget, *rect);
                        }
                        datetime.next_hour()
                    }
                }
                if !event_is_set && context_selection.contexts[0].name != "General" {
                    date_selection.set_event(None);
                    let mut context = Context::general();
                    context.populate(task_list.clone());
                    context_selection.contexts[0] = context;
                }
                f.render_widget(schedule, rects[2]);

                // Task List
                // let list_items: Vec<ListItem> = task_selection.items.iter().map(|i| ListItem::new(i.to_string())).collect();
                // let task_list = List::new(list_items).block(Block::default()
                //     .title(Spans::from(vec![ Span::styled("Contexts", Style::default().fg(Color::Blue)), ]))
                //     .title_alignment(Alignment::Center)
                //     .border_style(Style::default().fg(if mode_selection.mode == "Contexts".to_string() {Color::Red} else {Color::White}))
                //     .borders(Borders::ALL)
                // ).highlight_style(Style::default().fg(Color::Blue));
                // f.render_stateful_widget(task_list, rects[0], &mut task_selection.state);


                let contexts = Block::default()
                    .title(Spans::from(vec![ Span::styled("Contexts", Style::default().fg(Color::Blue)), ]))
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg( if mode_selection.mode == "Contexts".to_string() {Color::Red} else {Color::White}))
                    .borders(Borders::ALL);
                // for rect in create_context_rects(contexts.inner(rects[0]), show_keys, context_list.len()+1) {
                let mut context_rects = create_context_rects(contexts.inner(rects[0]), show_keys, context_selection.len());
                // let extra_rect = context_rects[0];
                if context_selection.len() % 2 == 1 {
                    context_rects[0] = context_rects[0].union(context_rects[1]);
                    context_rects.remove(1);
                }
                // let rects = create_context_rects(contexts.inner(rects[0]), show_keys, context_list.len()+1);
                // println!("{}", context_selection.len());
                for (i, rect) in context_rects.iter().enumerate() {
                    if show_keys && i == context_rects.len()-1 {
                        f.render_widget(
                            Paragraph::new("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~")
                            .block(Block::default()
                                .borders(Borders::TOP)
                                .border_style(Style::default().fg( if mode_selection.mode == "Contexts".to_string() {Color::Red} else {Color::White}))
                        ), *rect);
                    } else {
                        let style = Style::default();
                        let mut list_items: Vec<ListItem> = Vec::new();
                        let mut hidden = 0;
                        for (j, task) in context_selection.contexts[i].tasks.iter().enumerate() {
                            let mut string = format!("{: <width$}{: >5}", task.description, task.urg, width=rect.width as usize-7);
                            if context_selection.show_annos {
                                if context_selection.all_annos {
                                    for annotation in &task.annotation { string += "\n"; string += annotation }
                                } else if !context_selection.all_annos && context_selection.index == i && Some(j) == context_selection.contexts[i].state.selected() {
                                    for annotation in &task.annotation { string += "\n"; string += annotation }
                                }                             }
                            if task.status != "deleted" && task.status != "recurring" && !(!show_completed && task.status != "pending") {
                                list_items.push(ListItem::new(string)
                                .style(if task.status != "pending" {style.add_modifier(Modifier::DIM)} else {style}));
                            } else { hidden += 1 }
                        }
                        context_selection.contexts[i].hidden = hidden;
                        let widget = List::new(list_items)
                            .block(Block::default()
                                .title(Spans::from(vec![ Span::styled(context_selection.contexts[i].name.clone(), Style::default().fg(Color::Blue)), ]))
                                .title_alignment(Alignment::Center)
                                .border_style(Style::default().fg( if context_selection.index==i && context_selection.is_sel() {Color::Red} else {Color::White}))
                                .borders(Borders::ALL))
                            .highlight_style(Style::default().fg(Color::Blue));
                            f.render_stateful_widget(widget, *rect, &mut context_selection.contexts[i].state);
                    }

                }
                f.render_widget(contexts, rects[0]);
            }
        })?;






        // Key Handling
        let x = it.next();
        let event = x.unwrap_or(Ok(Key::Null))?;
        match mode_selection.mode.as_str() {
            "Contexts" => match event {
                Key::BackTab => {context_selection.deselect(); for context in &mut context_selection.contexts { context.deselect() };}
                Key::Char('\t') => {context_selection.deselect(); for context in &mut context_selection.contexts { context.deselect() };}
                Key::Char('j') => {context_selection.next_task()}
                Key::Char('k') => {context_selection.prev_task()}
                Key::Char('J') => {context_selection.next(); context_selection.select(); for context in &mut context_selection.contexts { context.deselect() }},
                Key::Char('K') => {context_selection.prev(); context_selection.select(); for context in &mut context_selection.contexts { context.deselect() }},
                Key::Char(' ') => {context_selection.toggle_show_annos()},
                Key::Char('A') => {context_selection.toggle_all_annos()},
                Key::Char('c') => if let Some(task) = context_selection.task() { task.toggle_complete() },
                Key::Char('.') => show_completed = !show_completed,
                _ => (),
            }
            "Schedule" => {
                match mode_selection.leader {
                    Key::Char('g') => match event {
                        Key::Char('h') => {date_selection.set_time(time); date_selection.set_date(today); mode_selection.reset_leader() },
                        Key::Char('0') => {date_selection.set_time(NaiveTime::from_hms(0, 0, 0)); mode_selection.reset_leader() },
                        Key::Null => (),
                        _ => mode_selection.reset_leader(),
                    }
                    Key::Null => match event {
                        Key::BackTab => {context_selection.select()}
                        Key::Char('j') => {date_selection.next_hour()},
                        Key::Char('k') => {date_selection.prev_hour()},
                        Key::Char('l') => date_selection.next_day(),
                        Key::Char('h') => date_selection.prev_day(),
                        Key::Char('J') => {date_selection.next_week()},
                        Key::Char('K') => {date_selection.prev_week()},
                        Key::Char('L') => date_selection.next_month(),
                        Key::Char('H') => date_selection.prev_month(),
                        Key::Char('g') => mode_selection.leader = Key::Char('g'),
                        _ => (),
                    }
                    _ => (),
                }
            }
            "Calendar" => {
                match mode_selection.leader {
                    Key::Char('g') => match event {
                        Key::Char('h') => {date_selection.set_date(today); mode_selection.reset_leader()},
                        Key::Char('0') => {date_selection.set_date(NaiveDate::from_ymd(date_selection.year(),1,1)); mode_selection.reset_leader()},
                        Key::Null => (),
                        _ => mode_selection.reset_leader(),
                    }
                    Key::Null => match event {
                        Key::Char('\t') => {context_selection.select()}
                        Key::Char('t') => date_selection.set_date(Local::today().naive_local()),
                        Key::Char('L') => date_selection.next_year(),
                        Key::Char('H') => date_selection.prev_year(),
                        Key::Char('J') => date_selection.next_month(),
                        Key::Char('K') => date_selection.prev_month(),
                        Key::Char('j') => date_selection.next_week(),
                        Key::Char('k') => date_selection.prev_week(),
                        Key::Char('l') => date_selection.next_day(),
                        Key::Char('h') => date_selection.prev_day(),
                        // Key::Char('a') => task_selection.items.push(date_selection.date.to_string()),
                        // Key::Char('p') => task_selection.items.push(date_selection.day().to_string()),
                        Key::Char('g') => mode_selection.leader = Key::Char('g'),
                        _ => (),
                    }
                    _ => (),
                }
            }
            &_ => (),
        }
        match event {
            Key::Char('q') => break,
            Key::Char('\t') => mode_selection.next(),
            Key::BackTab => mode_selection.prev(),
            Key::Char('?') => show_keys = !show_keys,
            Key::Char('1') => mode_selection.contexts(),
            Key::Char('2') => mode_selection.schedule(),
            Key::Char('3') => mode_selection.calendar(),
            Key::Null => {thread::sleep(Duration::from_millis(10)); continue},
            _ => (),
        }
        if show_title {
            match event {
                Key::Null => (),
                _ => show_title = false,
            }
        }
    }
    Ok(())
}

fn create_context_rects(size: Rect, show_keys: bool, count: usize) -> Vec<Rect> {
    let cols = if (count+1)/2 < 2 { 2 } else {(count+1)/2};
    let mut rects: Vec<Rect> = Vec::new();
    let mut init_rects = vec!(size);
    if show_keys {
        init_rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints( vec!(Constraint::Min(5), Constraint::Length(2)) )
            .split(size);
    }
    let mid_rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints( vec!(Constraint::Percentage(100/cols as u16); cols ) )
                .split(init_rects[0]);
    if count >= 3 {
        for rect in mid_rects.iter() {
            let constraints = vec!(Constraint::Percentage(50); 2);
            rects.extend(Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(*rect));
        }
    } else { rects.extend(mid_rects) }
    if show_keys {
        rects.push(init_rects[1]);
    }
    // println!("{:?}", rects);
    rects
}

fn create_cal_rects(size: Rect, show_keys: bool) -> Vec<Rect> {
    let rows: usize = 7 + show_keys as usize;
    let init_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints( vec![ Constraint::Length(1); rows ].as_ref())
        .split(size);
    let mut rects: Vec<Rect> = Vec::new();
    for (i, rect) in init_rects.iter().enumerate() {
        if show_keys && i == init_rects.len()-1 { rects.push(*rect) }
        else{
            rects.extend(Layout::default()
                .direction(Direction::Horizontal)
                .constraints( [ Constraint::Length(4); 7 ].as_ref())
                .split(*rect));
        }
    }
    rects
}

fn create_sched_rects(size: Rect, show_keys: bool) -> (Vec<Rect>, Vec<Rect>) {
    // let rows: usize = ()(size.height/2) + show_keys as usize;
    let constraints = vec![ Constraint::Length(2); (size.height/2).into() ];
    let init_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints( constraints.as_ref())
        .split(size);
    let mut left_rects: Vec<Rect> = Vec::new();
    let mut right_rects: Vec<Rect> = Vec::new();
    for (i, rect) in init_rects.iter().enumerate() {
        if show_keys && i == init_rects.len()-1 { left_rects.push(*rect) }
        else{
            let rects = Layout::default()
                .direction(Direction::Horizontal)
                .constraints( [ Constraint::Length(3), Constraint::Min(3) ].as_ref())
                .split(*rect);
            left_rects.push(rects[0]);
            right_rects.push(rects[1]);
        }
    }
    (left_rects, right_rects)
}

fn create_rects(size: Rect, show_keys: bool) -> Vec<Rect> {
    let mut rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints( [ Constraint::Length(30), Constraint::Min(30), ].as_ref())
        .split(size);

    let subrects = Layout::default()
        .direction(Direction::Vertical)
        .constraints( [ Constraint::Length(9+2*show_keys as u16), Constraint::Min(30), ].as_ref())
        .split(rects.remove(0));
    rects.extend(subrects);
    rects
}

fn create_dates(sel_date: &DateSelection) -> Vec<NaiveDate> {
    let mut prev_month = DateSelection::date(sel_date.date); prev_month.prev_month();
    let curr_month = DateSelection::date(sel_date.date);
    let mut next_month = DateSelection::date(sel_date.date); next_month.next_month();

    let mut dates: Vec<NaiveDate> = Vec::new();
    for date in prev_month.month_length()-curr_month.first().weekday().num_days_from_sunday()-7..prev_month.month_length() { dates.push(NaiveDate::from_ymd(prev_month.year(), prev_month.month(), (date+1).into()))};
    for date in 0..curr_month.month_length() { dates.push(NaiveDate::from_ymd(curr_month.year(), curr_month.month(), (date+1).into())) };
    let offset = if dates.len() < 42 { 14 } else { 7 };
    for date in 0..offset-next_month.first().weekday().num_days_from_sunday() { dates.push(NaiveDate::from_ymd(next_month.year(), next_month.month(), (date+1).into()))};
    return dates
}

fn create_title(size: Rect) -> (Paragraph<'static>, Paragraph<'static>) {
    let mut substring = vec!(
        ":([\"               \"])".to_string() ,
        "`Y88ba,         ,ad88P".to_string()   ,
        " `88888ba     ad88888'".to_string()   ,
        "  `Y88888b, ,d88888P' ".to_string()   ,
        ":,__`\"Y888b,d888P\"'__,".to_string() ,
        "`Y88ba, ``\":\"'' ,ad88P".to_string() ,
        " `88888ba     ad88888'".to_string()   ,
        "  `Y88888b, ,d88888P' ".to_string()   ,
        "    `\"Y888b,d888P\"'   ".to_string() ,
        "        \"]):([\"       ".to_string() ,
        "     ,ad88P`Y88ba,    ".to_string()   ,
        "   ad88888' `88888ba  ".to_string()   ,
        " ,d88888P'   `Y88888b,".to_string()   ,
        ",d888P\"'__,:,__`\"Y888b".to_string() ,
        ":\"'' ,ad88P`Y88ba, ``\"".to_string() ,
        "   ad88888' `88888ba  ".to_string()   ,
        " ,d88888P'   `Y88888b,".to_string()   ,
        ",d888P\"'       `\"Y888b".to_string());

    let mut i = 0;
    let mut width = size.width/22;
    while width > 0 { width /= 2; i+=1; }
    for _ in 0..i { substring = substring.iter().map(|s| s.to_string()+s).collect::<Vec<String>>(); }
    let mid_string = substring.join("\n") + "\n";
    let mut string = "".to_string();
    for _ in 0..size.height/18+1 { string += &mid_string; }

    let background = Paragraph::new(string
    ).block(Block::default().borders(Borders::NONE));

    let foreground = Paragraph::new(
        "".to_owned() +
        "                           d8b                         d8b                  \n" +
        "   d8P                     ?88                         88P                  \n" +
        "d888888P                    88b                       d88                   \n" +
        "  ?88'   d888b8b   .d888b,  888  d88' d8888b d888b8b  888    88bd88b .d888b,\n" +
        "  88P   d8P' ?88   ?8b,     888bd8P' d8P' `Pd8P' ?88  ?88    88P'  ` ?8b,   \n" +
        "  88b   88b  ,88b    `?8b  d88888b   88b    88b  ,88b  88b  d88        `?8b \n" +
        "  `?8b  `?88P'`88b`?888P' d88' `?88b,`?888P'`?88P'`88b  88bd88'     `?888P' \n"
    ).block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Red)));
    (foreground, background)


}
