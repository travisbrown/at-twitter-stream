use at_twitter_stream::{cli, db::Mapping, error::Error, extract::extract_user_info};
use bzip2::read::MultiBzDecoder;
use clap::{crate_authors, crate_version, Parser};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use tar::Archive;
use zip::ZipArchive;

fn process<R: Read>(source: R) -> Result<Vec<(u64, String, String)>, Error> {
    let reader = BufReader::new(MultiBzDecoder::new(source));
    let mut result = Vec::with_capacity(512);

    for batch_res in extract_user_info(reader) {
        let mut batch = batch_res?;

        result.append(&mut batch);
    }

    Ok(result)
}

fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    let _ = cli::init_logging(opts.verbose);

    let ext = OsStr::new("bz2");

    match opts.command {
        SubCommand::UserInfo { path } => {
            let db = Mapping::new(opts.path)?;

            if path.ends_with("tar") {
                let file = File::open(path)?;
                let mut archive = Archive::new(file);
                log::info!("Opening archive");

                for entry_res in archive.entries()? {
                    let mut entry = entry_res?;
                    let path = entry.path()?;

                    log::info!("{}", path.to_string_lossy());
                    if path.extension() == Some(ext) {
                        log::info!("FILE: {}", entry.path()?.to_string_lossy());

                        for (id, screen_name, _) in process(&mut entry)? {
                            db.insert_pair(id, &screen_name)?;
                        }
                    }
                }
            } else if path.ends_with("zip") {
                let file = File::open(path)?;
                let mut archive = ZipArchive::new(file)?;
                log::info!("Opening archive");

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let file_name = file.name();

                    if file_name.ends_with("bz2") {
                        log::info!("FILE: {}", file_name);

                        for (id, screen_name, _) in process(&mut file)? {
                            db.insert_pair(id, &screen_name)?;
                        }
                    }
                }
            }
        }
        SubCommand::QueryUserId { value } => {
            let db = Mapping::new(opts.path)?;

            let screen_names = db.lookup_by_id(value)?;

            for screen_name in screen_names {
                println!("{}", screen_name);
            }
        }
        SubCommand::QueryScreenName { value } => {
            let db = Mapping::new(opts.path)?;

            let ids = db.lookup_by_screen_name(&value)?;

            for id in ids {
                println!("{}", id);
            }
        }
        SubCommand::Stats => {
            let db = Mapping::new(opts.path)?;

            println!(
                "Estimated total key count: {}",
                db.get_estimated_key_count()?
            );

            let (id_keys, screen_name_keys) = db.get_key_counts();

            println!(
                "User ID keys: {}\nScreen name keys: {}",
                id_keys, screen_name_keys
            );
        }
        SubCommand::Import => {
            let db = Mapping::new(opts.path)?;

            let stdin = std::io::stdin();
            for line_res in stdin.lock().lines() {
                let line = line_res?;

                let mut fields = line.split(',');
                let id = fields.next().unwrap().parse::<u64>().unwrap();
                let screen_name = fields.next().unwrap();

                db.insert_pair(id, screen_name)?;
            }
        }
    }

    Ok(())
}

#[derive(Parser)]
#[clap(name = "atjson", version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// Level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// Path to the RocksDB directory
    #[clap(short, long, default_value = "data/user-db")]
    path: String,
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    UserInfo { path: String },
    QueryScreenName { value: String },
    QueryUserId { value: u64 },
    Import,
    Stats,
}
