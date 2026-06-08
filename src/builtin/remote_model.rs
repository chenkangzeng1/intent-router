use crate::{Handler, HandlerResult};

/// 远程大模型 API 调用 Handler
///
/// 用户注入 API 客户端和模型配置
pub struct RemoteModelHandler<I, O, C> {
    call_fn: Box<dyn Fn(&I, &C) -> (O, f32) + Send + Sync>,
}

impl<I, O, C> RemoteModelHandler<I, O, C> {
    /// 创建 RemoteModelHandler
    ///
    /// call_fn: 调用远程 API，返回 (output, confidence)
    pub fn new<F>(call_fn: F) -> Self
    where
        F: Fn(&I, &C) -> (O, f32) + Send + Sync + 'static,
    {
        Self {
            call_fn: Box::new(call_fn),
        }
    }
}

impl<I, O, C> Handler<I, O, C> for RemoteModelHandler<I, O, C> {
    fn handle(&self, input: &I, ctx: &C) -> HandlerResult<O> {
        let (output, confidence) = (self.call_fn)(input, ctx);
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
    fn test_remote_model_handler() {
        let handler = RemoteModelHandler::new(|input: &String, _ctx: &()| {
            if input.contains("推理") {
                (Decision::Execute("reasoning_task".into()), 0.92)
            } else {
                (Decision::Reply("unknown".into()), 0.3)
            }
        });

        let result = handler.handle(&"需要深度推理".into(), &());
        assert_eq!(result.output, Decision::Execute("reasoning_task".into()));
        assert_eq!(result.confidence, 0.92);
    }

    #[test]
    fn test_remote_model_low_confidence() {
        let handler = RemoteModelHandler::new(|_: &String, _: &()| {
            (Decision::Reply("uncertain".into()), 0.1)
        });

        let result = handler.handle(&"test".into(), &());
        assert_eq!(result.confidence, 0.1);
    }
}
