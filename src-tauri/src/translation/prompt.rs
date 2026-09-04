//! 翻译 Prompt 组装。输出约束（只输出译文）内建于系统提示词；
//! 未来新增翻译模式（学术/润色/口语）只是模板差异，不影响架构。

pub fn system_prompt(target_language: &str) -> String {
    format!(
        "你是一个专业的翻译引擎。把用户输入的文本翻译成{target_language}。\n\
         要求：\n\
         1. 只输出译文本身，不要任何解释、注释或原文；\n\
         2. 保持原文的语气、格式与换行；\n\
         3. 专有名词、代码、URL、邮箱地址保持原样；\n\
         4. 译文要符合{target_language}的表达习惯，自然流畅。"
    )
}

pub fn user_prompt(source_text: &str, source_language: Option<&str>) -> String {
    match source_language {
        Some(lang) => format!("源语言：{lang}\n待翻译文本：\n{source_text}"),
        None => format!("请自动检测源语言。\n待翻译文本：\n{source_text}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_constrains_output_to_translation_only() {
        let prompt = system_prompt("英语");
        assert!(prompt.contains("只输出译文"));
        assert!(prompt.contains("英语"));
    }

    #[test]
    fn user_prompt_auto_detects_when_source_language_missing() {
        assert!(user_prompt("hello", None).contains("自动检测"));
        assert!(user_prompt("hello", Some("日语")).contains("日语"));
    }
}
