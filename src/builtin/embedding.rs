use crate::{Handler, HandlerResult};

/// Embedding 相似度计算 Handler
///
/// 用户需要注入 Embedding 客户端和向量库
pub struct EmbeddingHandler<I, O, C> {
    similarity_fn: Box<dyn Fn(&I, &C) -> Option<(O, f32)> + Send + Sync>,
    threshold: f32,
}

impl<I, O, C> EmbeddingHandler<I, O, C> {
    /// 创建 EmbeddingHandler
    ///
    /// similarity_fn: 计算输入与向量库的相似度，返回 (output, similarity_score)
    pub fn new<F>(similarity_fn: F) -> Self
    where
        F: Fn(&I, &C) -> Option<(O, f32)> + Send + Sync + 'static,
    {
        Self {
            similarity_fn: Box::new(similarity_fn),
            threshold: 0.85,
        }
    }

    /// 设置相似度阈值
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }
}

impl<I, O: Clone, C> Handler<I, O, C> for EmbeddingHandler<I, O, C> {
    fn handle(&self, input: &I, ctx: &C) -> HandlerResult<O> {
        match (self.similarity_fn)(input, ctx) {
            Some((output, score)) => {
                if score >= self.threshold {
                    HandlerResult::new(output, score)
                } else {
                    HandlerResult::new(output, 0.0)
                }
            }
            None => {
                panic!("EmbeddingHandler: similarity_fn returned None. Provide a fallback output in similarity_fn or handle in Router.")
            }
        }
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
    fn test_embedding_handler_above_threshold() {
        let handler = EmbeddingHandler::new(|input: &String, _ctx: &()| {
            if input.contains("docker") {
                Some((Decision::Execute("docker_cmd".into()), 0.95))
            } else {
                None
            }
        })
        .threshold(0.85);

        let result = handler.handle(&"如何使用docker".into(), &());
        assert_eq!(result.output, Decision::Execute("docker_cmd".into()));
        assert_eq!(result.confidence, 0.95);
    }

    #[test]
    fn test_embedding_handler_below_threshold() {
        let handler = EmbeddingHandler::new(|input: &String, _ctx: &()| {
            if input.contains("docker") {
                Some((Decision::Execute("docker_cmd".into()), 0.5))
            } else {
                None
            }
        })
        .threshold(0.85);

        let result = handler.handle(&"如何使用docker".into(), &());
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.output, Decision::Execute("docker_cmd".into()));
    }
}
