//! SSE 行解码：把字节流按行切分（处理 `\r\n` 与跨 chunk 半包），纯逻辑、可单测。

/// 增量喂入文本、输出完整行的解码器。
pub struct SseDecoder {
    buffer: String,
}

impl SseDecoder {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// 喂入一段文本，返回其中所有完整行（不含行尾换行符）。
    pub fn feed(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);
        let mut lines = Vec::new();
        while let Some(pos) = self.buffer.find('\n') {
            let line: String = self.buffer.drain(..=pos).collect();
            lines.push(line.trim_end_matches(['\r', '\n']).to_string());
        }
        lines
    }

    /// 流结束时取出没有换行结尾的残余内容（如有）。
    pub fn finish(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.buffer))
        }
    }
}

impl Default for SseDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_lines_across_chunk_boundaries() {
        let mut decoder = SseDecoder::new();
        assert_eq!(decoder.feed("data: {\"a\""), Vec::<String>::new());
        assert_eq!(
            decoder.feed(":1}\ndata: [DONE]\n"),
            vec!["data: {\"a\":1}".to_string(), "data: [DONE]".to_string()]
        );
    }

    #[test]
    fn handles_crlf_and_keepalive_comments() {
        let mut decoder = SseDecoder::new();
        assert_eq!(
            decoder.feed(": keepalive\r\nevent: ping\r\r\n"),
            vec![": keepalive".to_string(), "event: ping".to_string()]
        );
    }

    #[test]
    fn finish_flushes_trailing_line() {
        let mut decoder = SseDecoder::new();
        decoder.feed("data: tail");
        assert_eq!(decoder.finish(), Some("data: tail".to_string()));
        assert_eq!(decoder.finish(), None);
    }
}
