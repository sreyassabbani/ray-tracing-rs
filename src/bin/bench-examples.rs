use std::{
    env,
    error::Error,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const EXAMPLES: [&str; 2] = ["defocus-blur", "rtiow-final"];

struct Args {
    repeat: usize,
    example: Option<String>,
    skip_build: bool,
}

struct Metrics {
    example: String,
    repeats: usize,
    avg: Duration,
    min: Duration,
    max: Duration,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args()?;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let examples = selected_examples(args.example.as_deref())?;

    if !args.skip_build {
        build_examples(&manifest_dir)?;
    }

    let target_dir = manifest_dir.join("target").join("release").join("examples");
    let output_path = manifest_dir.join("output.ppm");

    let mut results = Vec::with_capacity(examples.len());
    for example in examples {
        results.push(benchmark_example(
            &manifest_dir,
            &target_dir,
            &output_path,
            example,
            args.repeat,
        )?);
    }

    println!();
    println!(
        "{:<16} {:>7} {:>12} {:>12} {:>12}",
        "example", "repeats", "avg", "min", "max"
    );
    for result in results {
        println!(
            "{:<16} {:>7} {:>12} {:>12} {:>12}",
            result.example,
            result.repeats,
            format_duration(result.avg),
            format_duration(result.min),
            format_duration(result.max),
        );
    }

    Ok(())
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
    let mut repeat = 1usize;
    let mut example = None;
    let mut skip_build = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--repeat" | "-n" => {
                let raw = args.next().ok_or("missing value for --repeat")?;
                repeat = raw.parse()?;
                if repeat == 0 {
                    return Err("--repeat must be greater than zero".into());
                }
            }
            "--example" | "-e" => {
                example = Some(args.next().ok_or("missing value for --example")?);
            }
            "--skip-build" => skip_build = true,
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }

    Ok(Args {
        repeat,
        example,
        skip_build,
    })
}

fn print_usage() {
    println!("Benchmark the actual example executables.");
    println!();
    println!("Usage:");
    println!("  cargo run --release --bin bench-examples -- [options]");
    println!();
    println!("Options:");
    println!("  -n, --repeat <count>   Number of timed runs per example (default: 1)");
    println!("  -e, --example <name>   Benchmark only one example");
    println!("      --skip-build       Assume release examples are already built");
    println!("  -h, --help             Show this help text");
    println!();
    println!("Examples:");
    for example in EXAMPLES {
        println!("  {example}");
    }
}

fn selected_examples(example: Option<&str>) -> Result<Vec<&'static str>, Box<dyn Error>> {
    match example {
        Some(name) => EXAMPLES
            .iter()
            .copied()
            .find(|candidate| *candidate == name)
            .map(|candidate| vec![candidate])
            .ok_or_else(|| format!("unknown example: {name}").into()),
        None => Ok(EXAMPLES.to_vec()),
    }
}

fn build_examples(manifest_dir: &Path) -> Result<(), Box<dyn Error>> {
    let cargo = cargo_binary();
    let status = Command::new(cargo)
        .current_dir(manifest_dir)
        .args(["build", "--release", "--examples"])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("failed to build release examples".into())
    }
}

fn benchmark_example(
    manifest_dir: &Path,
    target_dir: &Path,
    output_path: &Path,
    example: &str,
    repeat: usize,
) -> Result<Metrics, Box<dyn Error>> {
    let binary = target_dir.join(example_binary_name(example));
    if !binary.exists() {
        return Err(format!(
            "missing example binary: {} (run without --skip-build first)",
            binary.display()
        )
        .into());
    }

    let mut durations = Vec::with_capacity(repeat);
    for run in 0..repeat {
        println!("benchmarking {example} ({}/{})", run + 1, repeat);
        let _ = fs::remove_file(output_path);

        let start = Instant::now();
        let status = Command::new(&binary)
            .current_dir(manifest_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        let elapsed = start.elapsed();

        if !status.success() {
            return Err(format!("example `{example}` exited with status {status}").into());
        }

        if output_path.exists() {
            fs::remove_file(output_path)?;
        }

        durations.push(elapsed);
    }

    let total_secs: f64 = durations.iter().map(Duration::as_secs_f64).sum();
    let avg = Duration::from_secs_f64(total_secs / repeat as f64);
    let min = durations.iter().copied().min().expect("at least one duration");
    let max = durations.iter().copied().max().expect("at least one duration");

    Ok(Metrics {
        example: example.to_string(),
        repeats: repeat,
        avg,
        min,
        max,
    })
}

fn cargo_binary() -> OsString {
    env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn example_binary_name(example: &str) -> OsString {
    let mut name = OsString::from(example);
    name.push(std::env::consts::EXE_SUFFIX);
    name
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs_f64();
    if seconds >= 1.0 {
        format!("{seconds:.2}s")
    } else {
        format!("{:.2}ms", seconds * 1_000.0)
    }
}
