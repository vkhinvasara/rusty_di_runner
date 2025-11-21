"""
Rust-powered Azure Document Intelligence client with concurrent processing.

This module provides a high-performance client for Azure Document Intelligence API,
enabling concurrent batch processing of documents from URLs or local files with
round-robin load balancing across multiple Azure resources.
"""

from typing import Any, Optional

class Credentials:
    """
    Represents authentication credentials for Azure Document Intelligence API access.
    
    This class holds the necessary credentials to authenticate and connect
    to an Azure Document Intelligence endpoint.
    
    Attributes:
        endpoint (str): The Azure Document Intelligence endpoint URL
    """
    
    endpoint: str
    
    def __init__(self, endpoint: str, api_key: str) -> None:
        """
        Initialize credentials for Azure Document Intelligence.
        
        Args:
            endpoint: The Azure Document Intelligence endpoint URL 
                (e.g., "https://your-resource.cognitiveservices.azure.com")
            api_key: The API subscription key for authentication
        
        Example:
            >>> creds = Credentials(
            ...     endpoint="https://your-resource.cognitiveservices.azure.com",
            ...     api_key="your-api-key-here"
            ... )
        """
        ...

class RustyAnalysisClient:
    """
    A high-performance client for analyzing documents using Azure Document Intelligence API.
    
    This client provides batch processing capabilities with automatic round-robin
    load balancing across multiple Azure Document Intelligence resources. Documents
    are distributed evenly across all provided credentials for optimal throughput
    and resource utilization.
    
    The client uses Rust's async runtime (Tokio) for true concurrent processing,
    providing significant performance improvements over sequential processing.
    
    Features:
        - Concurrent batch processing of documents
        - Round-robin load balancing across multiple resources
        - Configurable rate limiting per resource
        - Support for both URL and file path inputs
        - Optional analysis features (OCR, formulas, fonts, etc.)
    """
    
    def __init__(self, credentials: list[Credentials], enable_logs=False) -> RustyAnalysisClient:
        """
        Create a new RustyAnalysisClient instance.
        
        Args:
            - **credentials**: List of Credentials objects containing endpoint URLs 
            and API keys for Azure Document Intelligence services. Documents 
            will be distributed across these resources in round-robin fashion.
            
            - **enable_logs**: Boolean flag to enable or disable logging output. 
            Defaults to False.
        
        Returns:
            A new client instance configured with the provided credentials.
        
        Raises:
            ValueError: If credentials list is empty or None.
        
        Example:
            >>> from rusty_di_runner import RustyAnalysisClient, Credentials
            >>> # Single resource
            >>> creds = [
            ...     Credentials(
            ...         endpoint="https://resource1.cognitiveservices.azure.com",
            ...         api_key="key1"
            ...     )
            ... ]
            >>> client = RustyAnalysisClient(credentials=creds)
            >>> 
            >>> # Multiple resources for load balancing
            >>> creds = [
            ...     Credentials(endpoint="https://resource1.cognitiveservices.azure.com", api_key="key1"),
            ...     Credentials(endpoint="https://resource2.cognitiveservices.azure.com", api_key="key2"),
            ...     Credentials(endpoint="https://resource3.cognitiveservices.azure.com", api_key="key3"),
            ... ]
            >>> client = RustyAnalysisClient(credentials=creds)
            >>> 
            >>> # Enable logging for debugging
            >>> client = RustyAnalysisClient(credentials=creds, enable_logs=True)
        """
        ...
    
    def process_batch_documents_from_urls(
        self,
        model_id: str,
        document_urls: list[str],
        features: Optional[list[str]] = None,
        output_format: Optional[str] = None,
        max_rps: int = 15
    ) -> list[dict[str, Any] | Exception]:
        """
        Process multiple documents from URLs concurrently with round-robin distribution.
        
        Analyzes a batch of documents accessible via URLs using the specified
        Document Intelligence model. Documents are automatically distributed across
        all configured resources in round-robin fashion (doc0→resource0, doc1→resource1, 
        doc2→resource2, doc3→resource0, etc.). All documents are processed in parallel
        for maximum throughput.
        
        Args:
            model_id: The Document Intelligence model ID to use for analysis.
                Common models include:
                - 'prebuilt-layout': Extract text, tables, and document structure
                - 'prebuilt-invoice': Extract invoice-specific fields
                - 'prebuilt-receipt': Extract receipt information
                - 'prebuilt-read': OCR for text extraction
                - 'prebuilt-document': General document analysis
                - Custom model IDs from your trained models
            
            document_urls: List of publicly accessible document URLs to analyze.
                URLs must be accessible without authentication.
            
            features: Optional list of analysis features to enable. Available features:
                - 'ocrHighResolution': High-resolution OCR for better accuracy
                - 'languages': Language detection
                - 'formulas': Mathematical formula extraction
                - 'styleFont': Font style detection (bold, italic, etc.)
                - 'barcodes': Barcode detection and extraction
                - 'keyValuePairs': Key-value pair extraction
                Defaults to None (uses model default features).
            
            output_format: Optional output content format. Valid values are:
                - 'text' (default): Plain text representation with line breaks
                - 'markdown': Markdown formatted output preserving document structure
                Defaults to 'text' if not specified.
            
            max_rps: Maximum requests per second per resource to control rate limiting.
                Helps respect Azure API quotas. Defaults to 15.
        
        Returns:
            List of results where each item corresponds to the input document at the
            same index. Each item is either:
                - dict: Successfully analyzed document result containing the full
                  analyzeResult with 'content', 'pages', 'tables', etc.
                - Exception: Error object if processing failed for that document
        
        Example:
            >>> urls = [
            ...     "https://example.com/invoice1.pdf",
            ...     "https://example.com/invoice2.pdf",
            ...     "https://example.com/invoice3.pdf"
            ... ]
            >>> 
            >>> # Basic usage
            >>> results = client.process_batch_documents_from_urls(
            ...     model_id="prebuilt-layout",
            ...     document_urls=urls
            ... )
            >>> 
            >>> # With optional features and custom rate limit
            >>> results = client.process_batch_documents_from_urls(
            ...     model_id="prebuilt-layout",
            ...     document_urls=urls,
            ...     features=['ocrHighResolution', 'formulas', 'languages'],
            ...     max_rps=20
            ... )
            >>> 
            >>> # Handle results
            >>> for i, result in enumerate(results):
            ...     if isinstance(result, Exception):
            ...         print(f"Document {i} failed: {result}")
            ...     else:
            ...         content = result.get('content', '')
            ...         pages = result.get('pages', [])
            ...         print(f"Document {i}: {len(pages)} pages, {len(content)} chars")
        
        Note:
            With multiple credentials, documents are distributed automatically:
            - 3 resources + 30 documents = 10 documents per resource
            - Document 0, 3, 6, 9... → Resource 0
            - Document 1, 4, 7, 10... → Resource 1
            - Document 2, 5, 8, 11... → Resource 2
        """
        ...
    
    def process_batch_documents_from_file_paths(
        self,
        model_id: str,
        file_paths: list[str],
        features: Optional[list[str]] = None,
        output_format: Optional[str] = None,
        max_rps: int = 15
    ) -> list[dict[str, Any] | Exception]:
        """
        Process multiple documents from local file paths concurrently with round-robin distribution.
        
        Analyzes a batch of local documents using the specified Document Intelligence
        model. Files are read from disk, distributed across all configured resources
        in round-robin fashion, and uploaded in parallel for maximum efficiency.
        
        Args:
            model_id: The Document Intelligence model ID to use for analysis.
                Common models include:
                - 'prebuilt-layout': Extract text, tables, and document structure
                - 'prebuilt-invoice': Extract invoice-specific fields
                - 'prebuilt-receipt': Extract receipt information
                - 'prebuilt-read': OCR for text extraction
                - 'prebuilt-document': General document analysis
                - Custom model IDs from your trained models
            
            file_paths: List of local file paths to process. Paths can be absolute
                or relative to the current working directory.
            
            features: Optional list of analysis features to enable. Available features:
                - 'ocrHighResolution': High-resolution OCR for better accuracy
                - 'languages': Language detection
                - 'formulas': Mathematical formula extraction
                - 'styleFont': Font style detection (bold, italic, etc.)
                - 'barcodes': Barcode detection and extraction
                - 'keyValuePairs': Key-value pair extraction
                Defaults to None (uses model default features).
            
            output_format: Optional output content format. Valid values are:
                - 'text' (default): Plain text representation with line breaks
                - 'markdown': Markdown formatted output preserving document structure
                Defaults to 'text' if not specified.
            
            max_rps: Maximum requests per second per resource to control rate limiting.
                Helps respect Azure API quotas. Defaults to 15.
        
        Returns:
            List of results where each item corresponds to the input file at the
            same index. Each item is either:
                - dict: Successfully analyzed document result containing the full
                  analyzeResult with 'content', 'pages', 'tables', etc.
                - Exception: Error object if processing failed for that document
        
        Supported file formats:
            - PDF (.pdf)
            - JPEG (.jpg, .jpeg)
            - PNG (.png)
            - TIFF (.tiff, .tif)
            - BMP (.bmp)
        
        Example:
            >>> file_paths = [
            ...     "C:/documents/invoice1.pdf",
            ...     "C:/documents/receipt2.jpg",
            ...     "/home/user/docs/form3.pdf"
            ... ]
            >>> 
            >>> # Basic usage
            >>> results = client.process_batch_documents_from_file_paths(
            ...     model_id="prebuilt-invoice",
            ...     file_paths=file_paths
            ... )
            >>> 
            >>> # With optional features
            >>> results = client.process_batch_documents_from_file_paths(
            ...     model_id="prebuilt-layout",
            ...     file_paths=file_paths,
            ...     features=['ocrHighResolution', 'formulas'],
            ...     max_rps=20
            ... )
            >>> 
            >>> # Handle results
            >>> successful = []
            >>> failed = []
            >>> for i, result in enumerate(results):
            ...     if isinstance(result, Exception):
            ...         failed.append((file_paths[i], result))
            ...     else:
            ...         successful.append(result)
            >>> 
            >>> print(f"Processed: {len(successful)} successful, {len(failed)} failed")
        
        Note:
            Files are distributed across resources in round-robin fashion just like URLs.
            Each file is read asynchronously and uploaded to its assigned resource
            in parallel with other files for optimal performance.
        
        Raises:
            Results list will contain Exception objects for files that couldn't be
            read or processed, but the function itself won't raise exceptions.
        """
        ...
