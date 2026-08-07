use std::path::PathBuf;

pub const DEFAULT_FMA_ROOT: &str = "/Users/tom/Downloads/FMA";
pub const DEFAULT_PER_FAMILY: usize = 5;
pub const DEFAULT_REVIEW_SEED_PER_FAMILY: usize = 2;
pub const DEFAULT_OUTPUT: &str = "target/stretch-corpus-fma-selection.md";

#[derive(Debug, PartialEq, Eq)]
pub struct SelectorArgs {
    pub fma_root: PathBuf,
    pub metadata: PathBuf,
    pub output: PathBuf,
    pub tsv_output: Option<PathBuf>,
    pub review_seed_tsv_output: Option<PathBuf>,
    pub per_family: usize,
    pub review_seed_per_family: usize,
}

impl Default for SelectorArgs {
    fn default() -> Self {
        let fma_root = PathBuf::from(DEFAULT_FMA_ROOT);
        Self {
            metadata: fma_root.join("fma_metadata/raw_tracks.csv"),
            fma_root,
            output: PathBuf::from(DEFAULT_OUTPUT),
            tsv_output: None,
            review_seed_tsv_output: None,
            per_family: DEFAULT_PER_FAMILY,
            review_seed_per_family: DEFAULT_REVIEW_SEED_PER_FAMILY,
        }
    }
}

pub fn parse_args<I>(args: I) -> Result<SelectorArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut parsed = SelectorArgs::default();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--fma-root" => {
                parsed.fma_root = PathBuf::from(next_value(&mut iter, "--fma-root")?);
                parsed.metadata = parsed.fma_root.join("fma_metadata/raw_tracks.csv");
            }
            "--metadata" => {
                parsed.metadata = PathBuf::from(next_value(&mut iter, "--metadata")?);
            }
            "--output" => {
                parsed.output = PathBuf::from(next_value(&mut iter, "--output")?);
            }
            "--tsv-output" => {
                parsed.tsv_output = Some(PathBuf::from(next_value(&mut iter, "--tsv-output")?));
            }
            "--review-seed-tsv-output" => {
                parsed.review_seed_tsv_output = Some(PathBuf::from(next_value(
                    &mut iter,
                    "--review-seed-tsv-output",
                )?));
            }
            "--per-family" => {
                parsed.per_family = next_value(&mut iter, "--per-family")?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --per-family value: {error}"))?;
                if parsed.per_family == 0 {
                    return Err("--per-family must be greater than zero".to_string());
                }
            }
            "--review-seed-per-family" => {
                parsed.review_seed_per_family = next_value(&mut iter, "--review-seed-per-family")?
                    .parse::<usize>()
                    .map_err(|error| format!("invalid --review-seed-per-family value: {error}"))?;
                if parsed.review_seed_per_family == 0 {
                    return Err("--review-seed-per-family must be greater than zero".to_string());
                }
            }
            unknown => {
                return Err(format!("unknown argument: {unknown}"));
            }
        }
    }
    Ok(parsed)
}

fn next_value<I>(iter: &mut I, name: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing value for {name}"))
}

pub fn usage() -> &'static str {
    "usage: fma-stretch-corpus-select [--fma-root PATH] [--metadata CSV] [--per-family N] [--output PATH] [--tsv-output PATH] [--review-seed-tsv-output PATH] [--review-seed-per-family N]"
}
