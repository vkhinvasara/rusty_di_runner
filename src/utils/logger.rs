use once_cell::sync::OnceCell;
use tracing_subscriber::fmt;

static TRACING_INIT: OnceCell<()> = OnceCell::new();
pub fn init_tracing() {
    TRACING_INIT.get_or_init(|| {
        // Set up the formatting (e.g., colorful output)
        let subscriber = fmt::Subscriber::builder()
            // We use 'with_writer(std::io::stderr)' for better compatibility
            // with server logging systems.
            .with_writer(std::io::stderr)
            .finish();

        // Set the global default subscriber
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global tracing subscriber");
    });
}
