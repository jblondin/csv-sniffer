/*!
CSV metadata types.
*/
use std::fmt;
use std::path::Path;
use std::io::{Read, Seek};
use std::fs::File;

use csv::{Reader, ReaderBuilder, Terminator};

use error::*;
use field_type::Type;
use snip::snip_preamble;

/// Primary CSV metadata. Generated by
/// [`Sniffer::sniff_path`](../struct.Sniffer.html#method.sniff_path) or
/// [`Sniffer::sniff_reader`](../struct.Sniffer.html#method.sniff_reader) after examining a CSV
/// file.
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    /// [`Dialect`](struct.Dialect.html) subtype.
    pub dialect: Dialect,
    /// (Maximum) number of fields per record.
    pub num_fields: usize,
    /// Inferred field types.
    pub types: Vec<Type>,
}
impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Metadata")?;
        writeln!(f, "========")?;
        writeln!(f, "{}", self.dialect)?;
        writeln!(f, "Number of fields: {}", self.num_fields)?;
        writeln!(f, "Types:")?;
        for (i, ty) in self.types.iter().enumerate() {
            writeln!(f, "\t{}: {}", i, ty)?;
        }
        Ok(())
    }
}

/// Dialect-level metadata. This type encapsulates the details to be used to derive a
/// `ReaderBuilder` object (in the [`csv`](https://docs.rs/csv) crate).
///
/// Not all components of this type are currently detected by the sniffer, and may be detected in
/// the future.
#[derive(Clone)]
pub struct Dialect {
    /// CSV delimiter (field separator). Detected by sniffer.
    pub delimiter: u8,
    /// [`Header`](struct.Header.html) subtype (header row boolean and number of preamble rows).
    /// Detected by sniffer.
    pub header: Header,
    /// Record terminator. Currently not detected by sniffer; defaults to `Terminator::CRLF`.
    pub terminator: Terminator,
    /// Record quoting details. Detected by sniffer.
    pub quote: Quote,
    /// Whether or not doubled quotes are interpreted as escapes. Currently not detected by sniffer;
    /// defaults to `true`.
    pub doublequote_escapes: bool,
    /// Character used as escape, if any. Currently not detected by sniffer; defaults to
    /// `Escape::Disabled` (to escape a quote, use double quotes).
    pub escape: Escape,
    /// Character used as comment, if any. Currently not detected by sniffer; defaults to
    /// `Comment::Disabled`.
    pub comment: Comment,
    /// Whether or not the number of fields in a record is allowed to change. Detected by sniffer.
    pub flexible: bool,
}
impl PartialEq for Dialect {
    fn eq(&self, other: &Dialect) -> bool {
        self.delimiter == other.delimiter
            && self.header == other.header
            && match (self.terminator, other.terminator) {
                (Terminator::CRLF, Terminator::CRLF) => true,
                (Terminator::Any(left), Terminator::Any(right)) => left == right,
                _ => false
            }
            && self.quote == other.quote
            && self.doublequote_escapes == other.doublequote_escapes
            && self.escape == other.escape
            && self.comment == other.comment
            && self.flexible == other.flexible
    }
}
impl fmt::Debug for Dialect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Dialect")
            .field("delimiter", &char::from(self.delimiter))
            .field("header", &self.header)
            .field("terminator", &self.terminator)
            .field("quote", &self.quote)
            .field("doublequote_escapes", &self.doublequote_escapes)
            .field("escape", &self.escape)
            .field("comment", &self.comment)
            .field("flexible", &self.flexible)
            .finish()
    }
}
impl fmt::Display for Dialect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Dialect:")?;
        writeln!(f, "\tDelimiter: {}", char::from(self.delimiter))?;
        writeln!(f, "\tHas header row?: {}", self.header.has_header_row)?;
        writeln!(f, "\tNumber of preamble rows: {}", self.header.num_preamble_rows)?;
        writeln!(f, "\tQuote character: {}", match self.quote {
            Quote::Some(chr) => format!("{}", char::from(chr)),
            Quote::None => "none".into()
        })?;
        writeln!(f, "\tDouble-quote escapes?: {}", self.doublequote_escapes)?;
        writeln!(f, "\tEscape character: {}", match self.escape {
            Escape::Enabled(chr) => format!("{}", char::from(chr)),
            Escape::Disabled => "none".into(),

        })?;
        writeln!(f, "\tComment character: {}", match self.comment {
            Comment::Enabled(chr) => format!("{}", char::from(chr)),
            Comment::Disabled => "none".into()
        })?;
        writeln!(f, "\tFlexible: {}", self.flexible)
    }
}
impl Dialect {
    /// Use this `Dialect` to open a file specified by provided path. Returns a `Reader` (from the
    /// [`csv`](https://docs.rs/csv) crate). Fails on file opening or reading errors.
    pub fn open_path<P: AsRef<Path>>(&self, path: P) -> Result<Reader<File>> {
        self.open_reader(File::open(path)?)
    }

    /// Use this `Dialect` to create a `Reader` (from the [`csv`](https://docs.rs/csv) crate) using
    /// the provided reader. Fails if unable to read from the reader.
    pub fn open_reader<R: Read + Seek>(&self, mut rdr: R) -> Result<Reader<R>> {
        snip_preamble(&mut rdr, self.header.num_preamble_rows)?;
        let bldr: ReaderBuilder = self.clone().into();
        Ok(bldr.from_reader(rdr))
    }
}
impl From<Dialect> for ReaderBuilder {
    fn from(dialect: Dialect) -> ReaderBuilder {
        let mut bldr = ReaderBuilder::new();
        bldr.delimiter(dialect.delimiter)
            .has_headers(dialect.header.has_header_row)
            .terminator(dialect.terminator)
            .escape(dialect.escape.into())
            .double_quote(dialect.doublequote_escapes)
            .comment(dialect.comment.into())
            .flexible(dialect.flexible);

        match dialect.quote {
            Quote::Some(character) => {
                bldr.quoting(true);
                bldr.quote(character);
            },
            Quote::None => {
                bldr.quoting(false);
            }
        }

        bldr
    }
}

/// Metadata about the header of the CSV file.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Whether or not this CSV file has a header row (a row containing column labels).
    pub has_header_row: bool,
    /// Number of rows that occur before either the header row (if `has_header_row` is `true), or
    /// the first data row.
    pub num_preamble_rows: usize
}

/// Metadata about the quoting style of the CSV file.
#[derive(Clone, PartialEq)]
pub enum Quote {
    /// Quotes are not used in the CSV file.
    None,
    /// Quotes are enabled, with the provided character used as the quote character.
    Some(u8)
}
impl fmt::Debug for Quote {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Quote::Some(ref character) => {
                f.debug_struct("Some")
                    .field("character", &char::from(*character))
                    .finish()
            },
            Quote::None => write!(f, "None")
        }
    }
}

/// The escape character (or `Disabled` if escaping is disabled)
#[derive(Clone, PartialEq)]
pub enum Escape {
    /// Escapes are enabled, with the provided character as the escape character.
    Enabled(u8),
    /// Escapes are disabled.
    Disabled
}
impl From<Escape> for Option<u8> {
    fn from(escape: Escape) -> Option<u8> {
        match escape {
            Escape::Enabled(chr) => Some(chr),
            Escape::Disabled => None
        }
    }
}
impl fmt::Debug for Escape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Escape::Enabled(chr) => write!(f, "Enabled({})", char::from(chr)),
            Escape::Disabled => write!(f, "Disabled")
        }
    }
}

/// The comment character (or `Disabled` if commenting doesn't exist in this dialect)
#[derive(Clone, PartialEq)]
pub enum Comment {
    /// Comments are enabled, with the provided character as the comment character.
    Enabled(u8),
    /// Comments are disabled.
    Disabled
}
impl From<Comment> for Option<u8> {
    fn from(comment: Comment) -> Option<u8> {
        match comment {
            Comment::Enabled(chr) => Some(chr),
            Comment::Disabled => None
        }
    }
}
impl fmt::Debug for Comment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Comment::Enabled(chr) => write!(f, "Enabled({})", char::from(chr)),
            Comment::Disabled => write!(f, "Disabled")
        }
    }
}
