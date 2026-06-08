use crate::{Handler, HandlerResult};

/// 本地轻量模型推理 Handler
///
/// 用户注入模型路径和 Prompt 模板
pub struct LocalModelHandler<I, O, C> {
    inference_fn: Box<dyn Fn(&I, &C) -> (O, f32) + Send + Sync>,
}

impl<I, O, C> LocalModelHandler<I, O, C> {
    /// 创建 LocalModelHandler
    ///
    /// inference_fn: 执行本地模型推理，返回 (output, confidence)
    pub fn new<F>(inference_fn: F) -> Self
    where
        F: Fn(&I, &C) -> (O, f32) + Send + Sync + 'static,
    {
        Self {
            inference_fn: Box::new(inference_fn),
        }
    }
}

impl<I, O, C> Handler<I, O, C> for LocalModelHandler<I, O, C> {
    fn handle(&self, input: &I, ctx: &C) -> HandlerResult<O> {
        let (output, confidence) = (self.inference_fn)(input, ctx);
        HandlerResult::new(output, confidence.clamp(0.0, 1.0))
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
    fn test_local_model_handler() {
        let handler = LocalModelHandler::new(|input: &String, _ctx: &()| {
            if input.contains("复杂") {
                (Decision::Execute("complex_task".into()), 0.85)
            } else {
                (Decision::Reply("unknown".into()), 0.2)
            }
        });

        let result = handler.handle(&"这是一个复杂任务".into(), &());
        assert_eq!(result.output, Decision::Execute("complex_task".into()));
        assert_eq!(result.confidence, 0.85);
    }

    #[test]
    fn test_local_model_confidence_clamping() {
        let handler = LocalModelHandler::new(|_: &String, _: &()| {
            (Decision::Reply("test".into()), 1.5)
        });

        let result = handler.handle(&"test".into(), &());
        assert_eq!(result.confidence, 1.0);
    }
}
