// 把块分成小块的方法
// 3个参数 => 输入的文本，块的大小，重叠区域的大小
pub fn fixed_length_chunking(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    assert!(chunk_size > overlap);

    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());

        let piece: String = chars[start..end].iter().collect();

        if !piece.is_empty() {
            chunks.push(piece);
        }
        if end == chars.len() {
            break;
        }
        start = end - overlap;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use crate::knowledge_base::chunk::fixed_length_chunking;

    #[test]
    fn test_basic_chunking() {
        let text = "abcddfadfafdaffadggg";
        let ret = fixed_length_chunking(text, 10, 2);
        assert_eq!(ret, vec!["abcddfadfa", "fafdaffadg", "dggg"])
    }

    #[test]
    fn test_chinese_chunking() {
        let text = "中国就打发法大大啊短发 大放大大的的打发斯";
        let ret = fixed_length_chunking(text, 6, 1);
        assert_eq!(
            ret,
            vec![
                "中国就打发法",
                "法大大啊短发",
                "发 大放大大",
                "大的的打发斯"
            ]
        )
    }

    #[test]
    fn test_empty_text() {
        let ret = fixed_length_chunking("", 10, 2);
        assert_eq!(ret, Vec::<String>::new());
    }

    #[test]
    fn test_text_shorter_than_chunk_size() {
        let text = "你好";
        let ret = fixed_length_chunking(text, 10, 2);
        assert_eq!(ret, vec!["你好"]);
    }

    #[test]
    fn test_zero_overlap() {
        let text = "abcdefghij";
        let ret = fixed_length_chunking(text, 3, 0);
        assert_eq!(
            ret,
            vec![
                "abc".to_string(),
                "def".to_string(),
                "ghi".to_string(),
                "j".to_string()
            ]
        );
    }
}
