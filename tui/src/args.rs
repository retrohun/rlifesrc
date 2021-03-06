//! Parsing command-line arguments.

use clap::{App, AppSettings, Arg, Error, ErrorKind, Result as ClapResult};
use rlifesrc_lib::{rules::NtLifeGen, Config, NewState, Search, SearchOrder, Symmetry, Transform};

fn is_positive(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit()) && s != "0" && !s.starts_with('-')
}

/// A struct to store the parse results.
pub(crate) struct Args {
    pub(crate) search: Box<dyn Search>,
    pub(crate) all: bool,
    #[cfg(feature = "tui")]
    pub(crate) reset: bool,
    #[cfg(feature = "tui")]
    pub(crate) no_tui: bool,
}

impl Args {
    /// Parses the command-line arguments.
    pub(crate) fn parse() -> ClapResult<Self> {
        let mut app = App::new("rlifesrc")
            .about("Searching for patterns in Conway's Game of Life")
            .long_about(
                "Searching for patterns in Conway's Game of Life\n\
                 \n\
                 The program is based on David Bell's lifesrc and Jason Summers's \n\
                 WinLifeSearch, using an algorithm invented by Dean Hickerson.\n\
                 \n\
                 The search result is displayed in a mix of Plaintext and \n\
                 RLE format.\n\
                 * Dead cells are represented by `.`;\n\
                 * Living cells are represented by `o` for rules with 2 states, \
                 `A` for rules with more states;\n\
                 * Dying cells are represented by uppercase letters starting from `B`;\n\
                 * Unknown cells are represented by `?`;\n\
                 * Each line is ended with `$`;\n\
                 * The whole pattern is ended with `!`",
            )
            .version("0.3.0")
            .settings(&[AppSettings::AllowNegativeNumbers, AppSettings::ColoredHelp])
            .arg(
                Arg::with_name("X")
                    .help("Width of the pattern")
                    .required(true)
                    .index(1)
                    .validator(|x| {
                        if is_positive(&x) {
                            Ok(())
                        } else {
                            Err(String::from("width must be a positive integer"))
                        }
                    }),
            )
            .arg(
                Arg::with_name("Y")
                    .help("Height of the pattern")
                    .required(true)
                    .index(2)
                    .validator(|y| {
                        if is_positive(&y) {
                            Ok(())
                        } else {
                            Err(String::from("height must be a positive integer"))
                        }
                    }),
            )
            .arg(
                Arg::with_name("P")
                    .help("Period of the pattern")
                    .default_value("1")
                    .index(3)
                    .validator(|p| {
                        if is_positive(&p) {
                            Ok(())
                        } else {
                            Err(String::from("period must be a positive integer"))
                        }
                    }),
            )
            .arg(
                Arg::with_name("DX")
                    .help("Horizontal translation")
                    .default_value("0")
                    .index(4)
                    .validator(|d| d.parse::<isize>().map(|_| ()).map_err(|e| e.to_string())),
            )
            .arg(
                Arg::with_name("DY")
                    .help("Vertical translation")
                    .default_value("0")
                    .index(5)
                    .validator(|d| d.parse::<isize>().map(|_| ()).map_err(|e| e.to_string())),
            )
            .arg(
                Arg::with_name("TRANSFORM")
                    .help("Transformation of the pattern")
                    .long_help(
                        "Transformation of the pattern\n\
                         After the last generation, the pattern will return to \
                         the first generation, applying this transformation first, \
                         and then the translation defined by DX and DY.\n\
                         You may need to add quotation marks for some of the transformations.\n\
                         \"Id\" is the identical transformation.\n\
                         \"R\" means counterclockwise rotation.\n\
                         \"F\" means flipping (reflection) across an axis.\n",
                    )
                    .short("t")
                    .long("transform")
                    .takes_value(true)
                    .possible_values(&["Id", "R90", "R180", "R270", "F|", "F-", "F\\", "F/"])
                    .default_value("Id"),
            )
            .arg(
                Arg::with_name("SYMMETRY")
                    .help("Symmetry of the pattern")
                    .long_help(
                        "Symmetry of the pattern\n\
                         You may need to add quotation marks for some of the symmetries.\n\
                         The usages of these symmetries are the same as Oscar Cunningham's \
                         Logic Life Search.\n\
                         See [https://conwaylife.com/wiki/Symmetry] \n",
                    )
                    .short("s")
                    .long("symmetry")
                    .takes_value(true)
                    .possible_values(&[
                        "C1", "C2", "C4", "D2|", "D2-", "D2\\", "D2/", "D4+", "D4X", "D8",
                    ])
                    .default_value("C1"),
            )
            .arg(
                Arg::with_name("RULE")
                    .help("Rule of the cellular automaton")
                    .long_help(
                        "Rule of the cellular automaton\n\
                         Supports Life-like, isotropic non-totalistic, hexagonal, MAP rules, \
                         and their corresponding Generations rules.\n",
                    )
                    .short("r")
                    .long("rule")
                    .takes_value(true)
                    .default_value("B3/S23")
                    .validator(|d| {
                        d.parse::<NtLifeGen>()
                            .map(|_| ())
                            .map_err(|e| e.to_string())
                    }),
            )
            .arg(
                Arg::with_name("ORDER")
                    .help("Search order")
                    .long_help(
                        "Search order\n\
                         Row first or column first.\n",
                    )
                    .short("o")
                    .long("order")
                    .takes_value(true)
                    .possible_values(&["row", "column", "automatic", "r", "c", "a"])
                    .default_value("automatic"),
            )
            .arg(
                Arg::with_name("CHOOSE")
                    .help("How to choose a state for unknown cells\n")
                    .short("c")
                    .long("choose")
                    .takes_value(true)
                    .possible_values(&["dead", "alive", "random", "d", "a", "r"])
                    .default_value("alive"),
            )
            .arg(
                Arg::with_name("MAX")
                    .help("Upper bound of numbers of minimum living cells in all generations")
                    .long_help(
                        "Upper bound of numbers of minimum living cells in all generations\n\
                         If this value is set to 0, it means there is no limitation.\n",
                    )
                    .short("m")
                    .long("max")
                    .takes_value(true)
                    .default_value("0")
                    .validator(|d| d.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())),
            )
            .arg(
                Arg::with_name("FRONT")
                    .help(
                        "Force the first row or column to be nonempty\n\
                         Here 'front' means the first row or column to be searched, \
                         according to the search order.",
                    )
                    .short("f")
                    .long("front"),
            )
            .arg(
                Arg::with_name("REDUCE")
                    .help("Reduce the max cell count when a result is found")
                    .long_help(
                        "Reduce the max cell count when a result is found\n\
                         The new max cell count will be set to the cell count of \
                         the current result minus one.",
                    )
                    .long("reduce"),
            );

        #[cfg(feature = "tui")]
        {
            app = app
                .arg(
                    Arg::with_name("ALL")
                        .help("Prints all possible results instead of only the first one")
                        .long_help(
                            "Prints all possible results instead of only the first one\n\
                             Only useful when --no-tui is set.\n",
                        )
                        .short("a")
                        .long("all")
                        .requires("NOTUI"),
                )
                .arg(
                    Arg::with_name("RESET")
                        .help("Resets the time when starting a new search")
                        .long("reset-time")
                        .conflicts_with("NOTUI"),
                )
                .arg(
                    Arg::with_name("NOTUI")
                        .help("Starts searching immediately, without entering the TUI")
                        .short("n")
                        .long("no-tui"),
                );
        }

        #[cfg(not(feature = "tui"))]
        {
            app = app.arg(
                Arg::with_name("ALL")
                    .help("Searches for all possible pattern")
                    .long_help("Searches for all possible pattern")
                    .short("a")
                    .long("all"),
            );
        }

        let matches = app.get_matches_safe()?;

        let width = matches.value_of("X").unwrap().parse().unwrap();
        let height = matches.value_of("Y").unwrap().parse().unwrap();
        let period = matches.value_of("P").unwrap().parse().unwrap();

        let dx = matches.value_of("DX").unwrap().parse().unwrap();
        let dy = matches.value_of("DY").unwrap().parse().unwrap();

        let transform: Transform = matches.value_of("TRANSFORM").unwrap().parse().unwrap();
        let symmetry: Symmetry = matches.value_of("SYMMETRY").unwrap().parse().unwrap();

        if width != height {
            if transform.square_world() {
                return Err(Error::with_description(
                    &format!(
                        "The transformation '{:?}' is only valid for square worlds",
                        transform
                    ),
                    ErrorKind::InvalidValue,
                ));
            }
            if symmetry.square_world() {
                return Err(Error::with_description(
                    &format!(
                        "The symmetry '{:?}' is only valid for square worlds",
                        symmetry
                    ),
                    ErrorKind::InvalidValue,
                ));
            }
        }

        let all = matches.is_present("ALL");
        #[cfg(feature = "tui")]
        let reset = matches.is_present("RESET");
        #[cfg(feature = "tui")]
        let no_tui = matches.is_present("NOTUI");
        let search_order = match matches.value_of("ORDER").unwrap() {
            "row" | "r" => Some(SearchOrder::RowFirst),
            "column" | "c" => Some(SearchOrder::ColumnFirst),
            _ => None,
        };
        let new_state = match matches.value_of("CHOOSE").unwrap() {
            "dead" | "d" => NewState::ChooseDead,
            "alive" | "a" => NewState::ChooseAlive,
            "random" | "r" => NewState::Random,
            _ => NewState::ChooseAlive,
        };
        let max_cell_count = matches.value_of("MAX").unwrap().parse().unwrap();
        let max_cell_count = match max_cell_count {
            0 => None,
            i => Some(i),
        };
        let non_empty_front = matches.is_present("FRONT");
        let reduce_max = matches.is_present("REDUCE");

        let rule_string = matches.value_of("RULE").unwrap().to_string();

        let config = Config::new(width, height, period)
            .set_translate(dx, dy)
            .set_transform(transform)
            .set_symmetry(symmetry)
            .set_search_order(search_order)
            .set_new_state(new_state)
            .set_max_cell_count(max_cell_count)
            .set_non_empty_front(non_empty_front)
            .set_reduce_max(reduce_max)
            .set_rule_string(rule_string);

        let search = config.world().unwrap();

        Ok(Args {
            search,
            all,
            #[cfg(feature = "tui")]
            reset,
            #[cfg(feature = "tui")]
            no_tui,
        })
    }
}
