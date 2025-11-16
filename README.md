# Rusty DI Runner

A high-performance Python library for batch document processing using Azure Document Intelligence API, powered by Rust for maximum concurrency and speed.

## Features

- **🚀 High Performance**: Built with Rust and Tokio for true concurrent processing
- **📦 Batch Processing**: Process multiple documents simultaneously with optimal throughput
- **⚡ Rate Limiting**: Built-in rate limiting with configurable max requests per second
- **🔄 Flexible Input**: Support for both URLs and local file paths
- **⚙️ Advanced Features**: Optional analysis features (OCR high resolution, formulas, style fonts, etc.)
- **🛡️ Type Safety**: Rust-backed reliability with Python ease of use
- **📊 Multiple Formats**: PDF, JPEG, PNG, TIFF, BMP support

## Installation

### From PyPI (when published)
```bash
pip install rusty_di_runner
```

### Local Development
```bash
# Install in editable mode
pip install -e .

# Or use maturin for development
maturin develop --release
```

## Quick Start

```python
from rusty_di_runner import RustyAnalysisClient

# Initialize the client
client = RustyAnalysisClient(
    endpoint="https://your-resource.cognitiveservices.azure.com",
    api_key="your-api-key"
)

# Process documents from URLs
urls = [
    "https://example.com/invoice1.pdf",
    "https://example.com/invoice2.pdf"
]

results = client.process_batch_documents_from_urls(
    model_id="prebuilt-invoice",
    document_urls=urls,
    features=["languages"],  # Optional features
    max_rps=15  # Optional rate limiting (default: 15)
)

# Process local files
file_paths = ["./documents/doc1.pdf", "./documents/doc2.pdf"]

results = client.process_batch_documents_from_file_paths(
    model_id="prebuilt-layout",
    file_paths=file_paths,
    features=["ocrHighResolution", "formulas"],
    max_rps=20  # Adjust rate limit as needed
)

# Handle results
for i, result in enumerate(results):
    if isinstance(result, Exception):
        print(f"Document {i} failed: {result}")
    else:
        print(f"Document {i} analyzed successfully")
        print(f"Content: {result.get('content', '')[:100]}")
```

## API Reference

### RustyAnalysisClient

#### Constructor

```python
client = RustyAnalysisClient(endpoint: str, api_key: str)
```

**Parameters:**
- `endpoint` (str): Azure Document Intelligence endpoint URL
- `api_key` (str): Azure subscription key for authentication

#### process_batch_documents_from_urls()

Process multiple documents from URLs concurrently.

```python
results = client.process_batch_documents_from_urls(
    model_id: str,
    document_urls: list[str],
    features: list[str] | None = None,
    max_rps: int = 15
)
```

**Parameters:**
- `model_id` (str): Document Intelligence model ID (e.g., 'prebuilt-layout', 'prebuilt-invoice', 'prebuilt-read')
- `document_urls` (list[str]): List of publicly accessible document URLs
- `features` (list[str] | None): Optional analysis features
- `max_rps` (int): Maximum requests per second to control rate limiting (default: 15)

**Returns:**
- `list`: List of results where each item is either a dict (success) or Exception (failure)

#### process_batch_documents_from_file_paths()

Process multiple documents from local file paths concurrently.

```python
results = client.process_batch_documents_from_file_paths(
    model_id: str,
    file_paths: list[str],
    features: list[str] | None = None,
    max_rps: int = 15
)
```

**Parameters:**
- `model_id` (str): Document Intelligence model ID
- `file_paths` (list[str]): List of local file paths
- `features` (list[str] | None): Optional analysis features
- `max_rps` (int): Maximum requests per second to control rate limiting (default: 15)

**Returns:**
- `list`: List of results where each item is either a dict (success) or Exception (failure)

## Supported Models

- `prebuilt-layout`: Extract text, tables, and structure
- `prebuilt-invoice`: Extract invoice-specific fields
- `prebuilt-receipt`: Extract receipt information
- `prebuilt-read`: OCR for text extraction
- `prebuilt-document`: General document analysis
- Custom models: Use your trained model ID

## Supported Features

Optional features you can enable:

- `ocrHighResolution`: High-resolution OCR for better accuracy
- `languages`: Language detection
- `formulas`: Mathematical formula extraction
- `styleFont`: Font style detection (bold, italic, etc.)
- `barcodes`: Barcode detection and extraction
- `keyValuePairs`: Key-value pair extraction

## Supported File Formats

- PDF (`.pdf`)
- JPEG (`.jpg`, `.jpeg`)
- PNG (`.png`)
- TIFF (`.tiff`, `.tif`)
- BMP (`.bmp`)

## Performance

Rusty DI Runner leverages Rust's async runtime (Tokio) to process documents concurrently, providing significant performance improvements over sequential processing:

- **Concurrent Processing**: All documents are processed in parallel
- **Rate Limiting**: Configurable semaphore-based rate limiting to respect API quotas (default: 15 RPS)
- **Efficient I/O**: Async file reading and HTTP requests
- **Low Overhead**: Minimal Python GIL interaction
- **Memory Efficient**: Streaming file uploads

## Error Handling

The library returns exceptions as part of the result list rather than raising them, allowing you to handle failures gracefully:

```python
results = client.process_batch_documents_from_urls(model_id, urls)

successful = []
failed = []

for i, result in enumerate(results):
    if isinstance(result, Exception):
        failed.append((i, result))
    else:
        successful.append(result)

print(f"Successful: {len(successful)}, Failed: {len(failed)}")
```

## Development

### Prerequisites

- Rust (latest stable)
- Python 3.8+
- Maturin

### Building

```bash
# Build wheel
maturin build --release

# Install locally
maturin develop --release

# Run tests
python -m pytest
```

### Project Structure

```
rusty_di_runner/
├── src/
│   └── lib.rs          # Main Rust implementation
├── examples/
│   └── test.py         # Example usage
├── Cargo.toml          # Rust dependencies
├── pyproject.toml      # Python project configuration
└── README.md
```

## Requirements

- Python 3.8+
- Azure Document Intelligence endpoint and API key
- Internet connectivity for URL-based processing

## License

See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

For issues and questions, please open an issue on the GitHub repository.
