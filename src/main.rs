use std::io::{self, Read, StdoutLock, Write};
use std::time::Duration;

use clap::Parser;
use crossterm::cursor::MoveToColumn;
use crossterm::event::{
    DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement};
use crossterm::{execute, queue};
use terminput::parser::parse_event;
use terminput::{Event, KeyCode, KeyEventKind};

/// Report raw terminal inputs.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Options {
    /// keyboard enhancement flag - disambiguate escape codes
    #[arg(short, long)]
    disambiguate: bool,

    /// keyboard enhancement flag - report all keys as escape codes
    #[arg(short = 'e', long)]
    all_escape: bool,

    /// keyboard enhancement flag - report alternate keys
    #[arg(short, long)]
    alternate_keys: bool,

    /// keyboard enhancement flag - report event types
    #[arg(short = 't', long)]
    event_types: bool,

    /// enable all kitty keyboard enhancements
    #[arg(short = 'k', long)]
    all_kitty: bool,

    /// enable bracketed paste
    #[arg(short, long)]
    bracketed_paste: bool,

    /// report mouse events
    #[arg(short, long)]
    mouse: bool,

    /// report focus change events
    #[arg(short, long)]
    focus: bool,
}

fn main() -> io::Result<()> {
    let options = Options::parse();

    enable_raw_mode().unwrap();
    let mut stdout = io::stdout().lock();
    let supports_keyboard_enhancement = matches!(supports_keyboard_enhancement(), Ok(true));

    if options.mouse {
        queue!(stdout, EnableMouseCapture)?;
    }
    if options.bracketed_paste {
        queue!(stdout, EnableBracketedPaste)?;
    }
    if options.focus {
        queue!(stdout, EnableFocusChange)?;
    }

    let mut flags = KeyboardEnhancementFlags::empty();
    if options.disambiguate || options.all_kitty {
        flags |= KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES;
    }
    if options.all_escape || options.all_kitty {
        flags |= KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;
    }
    if options.alternate_keys || options.all_kitty {
        flags |= KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS;
    }
    if options.event_types || options.all_kitty {
        flags |= KeyboardEnhancementFlags::REPORT_EVENT_TYPES;
    }

    if supports_keyboard_enhancement {
        queue!(stdout, PushKeyboardEnhancementFlags(flags))?;
    }

    stdout.flush()?;

    let mut stdin = std::io::stdin().lock();

    let mut q_key_count = 0;
    loop {
        let mut buf = [0; 4096];
        let read = stdin.read(&mut buf)?;
        if read > 0 {
            let mut read_bytes = &buf[..read];
            let mut esc_seq_count = read_bytes.iter().filter(|b| **b == b'\x1B').count();
            if esc_seq_count > 1 {
                while esc_seq_count > 1 {
                    let next_esc = read_bytes[1..].iter().position(|b| *b == b'\x1B').unwrap() + 1;
                    execute!(stdout, MoveToColumn(0)).unwrap();

                    if let Some(Event::Key(key_event)) =
                        write_sequence(&mut stdout, &read_bytes[..next_esc])?
                    {
                        if key_event.kind == KeyEventKind::Press {
                            if key_event.code == KeyCode::Char('q') {
                                q_key_count += 1;
                            } else {
                                q_key_count = 0;
                            }
                        }
                    }
                    esc_seq_count -= 1;
                    read_bytes = &read_bytes[next_esc..];
                }
            } else if let Some(Event::Key(key_event)) = write_sequence(&mut stdout, read_bytes)? {
                if key_event.kind == KeyEventKind::Press {
                    if key_event.code == KeyCode::Char('q') {
                        q_key_count += 1;
                    } else {
                        q_key_count = 0;
                    }
                }
            }

            if q_key_count > 1 {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    queue!(
        stdout,
        DisableMouseCapture,
        DisableBracketedPaste,
        DisableFocusChange
    )?;
    if supports_keyboard_enhancement {
        queue!(stdout, PopKeyboardEnhancementFlags)?;
    }
    stdout.flush()?;
    disable_raw_mode()?;
    Ok(())
}

fn write_sequence(stdout: &mut StdoutLock, read_bytes: &[u8]) -> io::Result<Option<Event>> {
    writeln!(
        stdout,
        "{}",
        read_bytes
            .iter()
            .map(|b| {
                let s = String::from_utf8(vec![*b]);
                if let Ok(r) = s {
                    let c = r.chars().next().unwrap();
                    if c.is_ascii_punctuation() || c.is_ascii_alphanumeric() {
                        return r;
                    }
                }
                format!("{:#02x}", b)
            })
            .collect::<Vec<_>>()
            .join("")
    )?;
    execute!(stdout, MoveToColumn(0))?;
    let event = parse_event(read_bytes).ok().flatten();
    if let Some(event) = &event {
        writeln!(stdout, "{event:?}")?;
        execute!(stdout, MoveToColumn(0)).unwrap();
    }
    Ok(event)
}
