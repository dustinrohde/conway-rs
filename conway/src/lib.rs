#![macro_use]
#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
extern crate num_integer;
extern crate serde;
extern crate serde_json;

pub mod config;
pub mod game;
pub mod grid;
pub mod point;

pub use config::GameConfig;
pub use errors::*;
pub use game::{Game, View};
pub use grid::Grid;
pub use point::Point;

mod errors {
    error_chain! {
        errors {
            ParseGrid(s: String) {
                description("failed to parse Grid"),
                display("failed to parse Grid: {}", s),
            }
            ParsePoint(s: String) {
                description("failed to parse Point"),
                display("failed to parse Point: {}", s),
            }
            ParseArg(arg: &'static str, expected: &'static str) {
                description("failed to parse argument"),
                display("failed to parse argument '{}': expected {}", arg, expected),
            }
        }

        foreign_links {
            IO(::std::io::Error);
        }
    }

    impl Error {
        pub fn to_string_chain(&self) -> String {
            let mut output: ::std::io::Cursor<Vec<u8>> = ::std::io::Cursor::new(Vec::new());
            self.write_err_chain(&mut output);
            output.into_inner().into_iter().map(char::from).collect()
        }

        pub fn write_err_chain<T: ::std::io::Write>(&self, output: &mut T) {
            let errmsg = "Well, shit. Encountered an error while trying to write another error. \
                          Good luck trying to figure it out!";
            writeln!(output, "error: {}", self).expect(errmsg);

            for err in self.iter().skip(1) {
                writeln!(output, "caused by: {}", err).expect(errmsg);
            }

            if let Some(backtrace) = self.backtrace() {
                writeln!(output, "backtrace: {:?}", backtrace).expect(errmsg);
            }
        }
    }
}
