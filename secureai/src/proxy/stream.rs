use anyhow::{Result, Context};
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct SSEChunk {
    pub event_type: String,
    pub data: String,
    pub token_count: u32,
}

pub struct SSEStreamParser {
    tokenizer: tiktoken_rs::CoreBPE,
}

impl SSEStreamParser {
    pub fn new() -> Result<Self> {
        // Load tokenizer for gpt-3.5-turbo
        let tokenizer = tiktoken_rs::get_bpe_from_model("gpt-3.5-turbo")
            .context("Failed to load tokenizer")?;

        Ok(Self { tokenizer })
    }

    pub fn parse_chunk(&self, chunk: Bytes) -> Result<SSEChunk> {
        // Parse SSE format: "event: type\ndata: {...}\n\n"
        let text = String::from_utf8(chunk.to_vec())
            .context("Invalid UTF-8 in chunk")?;

        let mut event_type = "message".to_string();
        let mut data = String::new();

        for line in text.lines() {
            if let Some(evt) = line.strip_prefix("event: ") {
                event_type = evt.to_string();
            } else if let Some(d) = line.strip_prefix("data: ") {
                data.push_str(d);
            }
        }

        // Count tokens in the data portion
        let token_ids = self.tokenizer.encode_ordinary(&data);
        let token_count = token_ids.len() as u32;

        Ok(SSEChunk {
            event_type,
            data,
            token_count,
        })
    }

    pub fn count_tokens(&self, text: &str) -> u32 {
        let token_ids = self.tokenizer.encode_ordinary(text);
        token_ids.len() as u32
    }
}

pub struct SSEStreamInspector {
    token_window: Vec<String>,
    window_size: usize,
}

impl SSEStreamInspector {
    pub fn new(window_size: usize) -> Self {
        Self {
            token_window: Vec::with_capacity(window_size),
            window_size,
        }
    }

    pub fn add_tokens(&mut self, text: &str) {
        self.token_window.push(text.to_string());

        // Keep sliding window of chunks (not individual tokens)
        if self.token_window.len() > self.window_size {
            self.token_window.remove(0);
        }
    }

    pub fn get_window_content(&self) -> String {
        self.token_window.join("")
    }

    pub fn clear(&mut self) {
        self.token_window.clear();
    }

    pub fn window_size(&self) -> usize {
        self.token_window.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_parser_creation() {
        let parser = SSEStreamParser::new().expect("Failed to create parser");
        assert!(true); // If we got here, parser initialized
    }

    #[test]
    fn test_parse_sse_chunk() {
        let parser = SSEStreamParser::new().expect("Failed to create parser");

        let chunk_text = "event: message\ndata: hello world\n\n";
        let chunk = Bytes::from(chunk_text);

        let parsed = parser.parse_chunk(chunk).expect("Failed to parse chunk");

        assert_eq!(parsed.event_type, "message");
        assert_eq!(parsed.data, "hello world");
        assert!(parsed.token_count > 0);
    }

    #[test]
    fn test_token_counting() {
        let parser = SSEStreamParser::new().expect("Failed to create parser");

        let tokens_short = parser.count_tokens("hello");
        let tokens_long = parser.count_tokens("hello world this is a longer sentence with many tokens");

        assert!(tokens_long > tokens_short);
    }

    #[test]
    fn test_stream_inspector_creation() {
        let inspector = SSEStreamInspector::new(50);
        assert_eq!(inspector.window_size(), 0);
    }

    #[test]
    fn test_stream_inspector_add_tokens() {
        let mut inspector = SSEStreamInspector::new(3);

        inspector.add_tokens("chunk1");
        assert_eq!(inspector.window_size(), 1);

        inspector.add_tokens("chunk2");
        assert_eq!(inspector.window_size(), 2);

        inspector.add_tokens("chunk3");
        assert_eq!(inspector.window_size(), 3);

        inspector.add_tokens("chunk4");
        // Window should stay at size 3 (sliding window)
        assert_eq!(inspector.window_size(), 3);
    }

    #[test]
    fn test_stream_inspector_window_content() {
        let mut inspector = SSEStreamInspector::new(10);

        inspector.add_tokens("hello");
        inspector.add_tokens(" ");
        inspector.add_tokens("world");

        let content = inspector.get_window_content();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_stream_inspector_clear() {
        let mut inspector = SSEStreamInspector::new(10);

        inspector.add_tokens("hello");
        inspector.add_tokens("world");

        assert_eq!(inspector.window_size(), 2);

        inspector.clear();
        assert_eq!(inspector.window_size(), 0);
        assert_eq!(inspector.get_window_content(), "");
    }
}
