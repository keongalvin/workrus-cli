use std::{
    io::{self, Write},
    process::ExitCode,
};
use workrus::{
    app,
    cli::{self, ParseResult},
};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let requested_json = args.iter().any(|arg| arg == "--json");
    match cli::parse(args) {
        Ok(ParseResult::Help) => write_stdout(cli::USAGE),
        Ok(ParseResult::Version) => {
            write_stdout(&format!("workrus {}\n", env!("CARGO_PKG_VERSION")))
        }
        Ok(ParseResult::Command { json, command }) => match app::run(command, json) {
            Ok(text) => write_stdout(&text),
            Err(error) => report(error, json),
        },
        Err(error) => report(error, requested_json),
    }
}

fn write_stdout(text: &str) -> ExitCode {
    write_stdout_with_code(text, 0)
}

fn write_stdout_with_code(text: &str, completed_code: u8) -> ExitCode {
    let mut stdout = io::stdout().lock();
    ExitCode::from(write_output(&mut stdout, text, completed_code))
}

fn write_output(writer: &mut impl Write, text: &str, completed_code: u8) -> u8 {
    match writer.write_all(text.as_bytes()) {
        Ok(()) => completed_code,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => 0,
        Err(_) => 1,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ReportDestination {
    Stdout,
    Stderr,
}

fn render_report(error: &workrus::error::AppError, json: bool) -> (ReportDestination, String) {
    if let Some(ref result) = error.partial_result {
        if json {
            return (ReportDestination::Stdout, result.clone());
        }
        let separator = if result.ends_with('\n') { "" } else { "\n" };
        return (
            ReportDestination::Stderr,
            format!("{result}{separator}workrus: {error}\n"),
        );
    }
    let message = if json {
        format!(
            "{{\"error\":{{\"code\":\"{}\",\"message\":{}}}}}\n",
            error.kind.code(),
            serde_json::to_string(&error.to_string()).expect("error messages serialize")
        )
    } else {
        format!("workrus: {error}\n")
    };
    (ReportDestination::Stderr, message)
}

fn report(error: workrus::error::AppError, json: bool) -> ExitCode {
    let exit_code = error.kind.exit_code() as u8;
    let (destination, message) = render_report(&error, json);
    match destination {
        ReportDestination::Stdout => write_stdout_with_code(&message, exit_code),
        ReportDestination::Stderr => {
            let _ = io::stderr().lock().write_all(message.as_bytes());
            ExitCode::from(exit_code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct BrokenPipe;

    impl Write for BrokenPipe {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn successful_partial_output_preserves_failure_code() {
        let mut output = Vec::new();

        assert_eq!(write_output(&mut output, "partial", 1), 1);
        assert_eq!(output, b"partial");
    }

    #[test]
    fn human_partial_result_exposes_completed_work_on_stderr() {
        let error = workrus::error::AppError::partial("Linear failed", "partial".to_owned());

        let rendered = render_report(&error, false);

        assert_eq!(rendered.0, ReportDestination::Stderr);
        assert!(rendered.1.contains("partial"));
        assert!(rendered.1.contains("workrus: Linear failed"));
    }

    #[test]
    fn json_partial_result_remains_the_only_stdout_document() {
        let error = workrus::error::AppError::partial(
            "Linear failed",
            "{\"result\":\"partial_failure\"}".to_owned(),
        );

        let rendered = render_report(&error, true);

        assert_eq!(rendered.0, ReportDestination::Stdout);
        assert_eq!(rendered.1, "{\"result\":\"partial_failure\"}");
    }

    #[test]
    fn broken_pipe_remains_a_silent_success() {
        assert_eq!(write_output(&mut BrokenPipe, "partial", 1), 0);
    }
}
