use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    #[clap(flatten)]
    pub(crate) instrumentation: qcdn_tracing_setup::instrumentation::Instrumentation,
}
