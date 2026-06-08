use crate::{Handler, HandlerResult};
use regex::Regex;

/// 规则匹配 Handler，支持正则表达式匹配
pub struct RuleHandler<I, O, C> {
    rules: Vec<(Regex, Box<dyn Fn(&I, &C) -> O + Send + Sync>)>,
    default_confidence: f32,
}

impl<I, O, C> RuleHandler<I, O, C> {
    /// 创建空的 RuleHandler
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_confidence: 1.0,
        }
    }

    /// 添加一条正则规则
    pub fn with_rule<F>(mut self, pattern: &str, mapper: F) -> Self
    where
        F: Fn(&I, &C) -> O + Send + Sync + 'static,
    {
        let regex = Regex::new(pattern).expect("Invalid regex pattern");
        self.rules.push((regex, Box::new(mapper)));
        self
    }

    /// 设置默认置信度
    pub fn confidence(mut self, confidence: f32) -> Self {
        self.default_confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

impl<I, O, C> Default for RuleHandler<I, O, C> {
    fn default() -> Self {
        Self::new()
    }
}

/// 为 String 输入实现便捷方法
impl<O: Clone, C> RuleHandler<String, O, C> {
    /// 添加字符串包含规则
    pub fn with_contains<F>(mut self, keyword: &str, mapper: F) -> Self
    where
        F: Fn(&String, &C) -> O + Send + Sync + 'static,
    {
        let pattern = regex::escape(keyword);
        let regex = Regex::new(&pattern).expect("Invalid regex pattern");
        self.rules.push((regex, Box::new(mapper)));
        self
    }
}

impl<O: Clone, C> Handler<String, O, C> for RuleHandler<String, O, C> {
    fn handle(&self, input: &String, ctx: &C) -> HandlerResult<O> {
        for (regex, mapper) in &self.rules {
            if regex.is_match(input) {
                let output = mapper(input, ctx);
                return HandlerResult::new(output, self.default_confidence);
            }
        }
        panic!("RuleHandler: no rule matched and no fallback configured. Add a catch-all rule with with_rule(\".*\", ...) or handle fallback in Router.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Decision {
        Execute(String),
        Reply(String),
    }

    #[test]
    fn test_rule_handler_regex_match() {
        let handler = RuleHandler::<String, _, _>::new()
            .with_rule(r"安装.*docker", |_, _| Decision::Execute("install_docker".into()))
            .with_rule(r"删除", |_, _| Decision::Execute("delete".into()));

        let result = handler.handle(&"请帮我安装一下docker".into(), &());
        assert_eq!(result.output, Decision::Execute("install_docker".into()));
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_rule_handler_contains() {
        let handler = RuleHandler::new()
            .with_contains("安装", |_, _| Decision::Execute("install".into()))
            .with_contains("卸载", |_, _| Decision::Execute("uninstall".into()));

        let result = handler.handle(&"我要卸载软件".into(), &());
        assert_eq!(result.output, Decision::Execute("uninstall".into()));
    }

    #[test]
    fn test_rule_handler_confidence() {
        let handler = RuleHandler::new()
            .confidence(0.9)
            .with_contains("test", |_, _| Decision::Execute("test".into()));

        let result = handler.handle(&"test".into(), &());
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    #[should_panic(expected = "no rule matched")]
    fn test_rule_handler_no_match_panic() {
        let handler = RuleHandler::new()
            .with_contains("安装", |_, _| Decision::Execute("install".into()));

        handler.handle(&"hello".into(), &());
    }
}
