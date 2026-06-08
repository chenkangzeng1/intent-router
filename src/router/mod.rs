/// Handler 结果包装，包含输出和置信度
#[derive(Debug, Clone, PartialEq)]
pub struct HandlerResult<O> {
    pub output: O,
    pub confidence: f32,
}

impl<O> HandlerResult<O> {
    pub fn new(output: O, confidence: f32) -> Self {
        Self { output, confidence }
    }
}

/// 级联策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    /// 短路，第一个命中即返回（默认）
    #[default]
    FirstMatch,
    /// 全量，选 confidence 最高
    HighestScore,
    /// 加权投票合并结果
    WeightedVote,
}

/// Handler trait，用户自定义判断逻辑
pub trait Handler<I, O, C> {
    fn handle(&self, input: &I, ctx: &C) -> HandlerResult<O>;
}

/// Router：级联调度、阈值比较、短路控制
pub struct Router<I, O, C> {
    handlers: Vec<Box<dyn Handler<I, O, C>>>,
    fallback: Box<dyn Fn(&I, &C) -> O>,
    threshold: f32,
    strategy: RoutingStrategy,
}

impl<I, O: Clone, C> Router<I, O, C> {
    /// 创建新的 Router，必须提供 fallback 函数
    pub fn new(fallback: impl Fn(&I, &C) -> O + 'static) -> Self {
        Self {
            handlers: Vec::new(),
            fallback: Box::new(fallback),
            threshold: 0.5,
            strategy: RoutingStrategy::FirstMatch,
        }
    }

    /// 添加 Handler（链式调用）
    pub fn with(mut self, handler: impl Handler<I, O, C> + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    /// 设置置信度阈值
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// 设置级联策略
    pub fn strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// 执行路由
    pub fn route(&self, input: &I, ctx: &C) -> O {
        match self.strategy {
            RoutingStrategy::FirstMatch => self.route_first_match(input, ctx),
            RoutingStrategy::HighestScore => self.route_highest_score(input, ctx),
            RoutingStrategy::WeightedVote => self.route_weighted_vote(input, ctx),
        }
    }

    fn route_first_match(&self, input: &I, ctx: &C) -> O {
        for handler in &self.handlers {
            let result = handler.handle(input, ctx);
            if result.confidence >= self.threshold {
                return result.output;
            }
        }
        (self.fallback)(input, ctx)
    }

    fn route_highest_score(&self, input: &I, ctx: &C) -> O {
        let mut best: Option<(O, f32)> = None;
        for handler in &self.handlers {
            let result = handler.handle(input, ctx);
            if result.confidence >= self.threshold {
                if best.as_ref().map_or(true, |(_, c)| result.confidence > *c) {
                    best = Some((result.output, result.confidence));
                }
            }
        }
        match best {
            Some((output, _)) => output,
            None => (self.fallback)(input, ctx),
        }
    }

    fn route_weighted_vote(&self, input: &I, ctx: &C) -> O {
        // 默认实现：加权投票选择最高分的（简化版，用户可扩展）
        self.route_highest_score(input, ctx)
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

    struct RuleHandler {
        keyword: String,
        decision: Decision,
        confidence: f32,
    }

    impl Handler<String, Decision, ()> for RuleHandler {
        fn handle(&self, input: &String, _ctx: &()) -> HandlerResult<Decision> {
            if input.contains(&self.keyword) {
                HandlerResult::new(self.decision.clone(), self.confidence)
            } else {
                HandlerResult::new(Decision::Reply(input.clone()), 0.0)
            }
        }
    }

    fn fallback(input: &String, _ctx: &()) -> Decision {
        Decision::Reply(format!("fallback: {}", input))
    }

    #[test]
    fn test_first_match_short_circuit() {
        let router = Router::new(fallback)
            .with(RuleHandler {
                keyword: "安装".into(),
                decision: Decision::Execute("install".into()),
                confidence: 0.9,
            })
            .with(RuleHandler {
                keyword: "docker".into(),
                decision: Decision::Execute("docker".into()),
                confidence: 0.8,
            });

        let result = router.route(&"我要安装 docker".into(), &());
        assert_eq!(result, Decision::Execute("install".into()));
    }

    #[test]
    fn test_fallback_when_no_match() {
        let router = Router::new(fallback).with(RuleHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.9,
        });

        let result = router.route(&"hello world".into(), &());
        assert_eq!(result, Decision::Reply("fallback: hello world".into()));
    }

    #[test]
    fn test_threshold_filter() {
        let router = Router::new(fallback)
            .threshold(0.8)
            .with(RuleHandler {
                keyword: "安装".into(),
                decision: Decision::Execute("install".into()),
                confidence: 0.7, // 低于阈值
            });

        let result = router.route(&"我要安装".into(), &());
        assert_eq!(result, Decision::Reply("fallback: 我要安装".into()));
    }

    #[test]
    fn test_highest_score_strategy() {
        let router = Router::new(fallback)
            .strategy(RoutingStrategy::HighestScore)
            .with(RuleHandler {
                keyword: "安装".into(),
                decision: Decision::Execute("install".into()),
                confidence: 0.6,
            })
            .with(RuleHandler {
                keyword: "docker".into(),
                decision: Decision::Execute("docker".into()),
                confidence: 0.9,
            });

        let result = router.route(&"我要安装 docker".into(), &());
        assert_eq!(result, Decision::Execute("docker".into()));
    }

    #[test]
    fn test_zero_confidence_continue() {
        let router = Router::new(fallback)
            .with(RuleHandler {
                keyword: "安装".into(),
                decision: Decision::Execute("install".into()),
                confidence: 0.0,
            })
            .with(RuleHandler {
                keyword: "docker".into(),
                decision: Decision::Execute("docker".into()),
                confidence: 0.9,
            });

        let result = router.route(&"我要安装 docker".into(), &());
        assert_eq!(result, Decision::Execute("docker".into()));
    }

    #[test]
    fn test_multiple_handlers_chain() {
        let router = Router::new(fallback)
            .with(RuleHandler {
                keyword: "a".into(),
                decision: Decision::Execute("a".into()),
                confidence: 0.3,
            })
            .with(RuleHandler {
                keyword: "b".into(),
                decision: Decision::Execute("b".into()),
                confidence: 0.4,
            })
            .with(RuleHandler {
                keyword: "c".into(),
                decision: Decision::Execute("c".into()),
                confidence: 0.9,
            });

        let result = router.route(&"abc".into(), &());
        assert_eq!(result, Decision::Execute("c".into()));
    }

    #[test]
    fn test_empty_router_fallback() {
        let router: Router<String, Decision, ()> = Router::new(fallback);
        let result = router.route(&"anything".into(), &());
        assert_eq!(result, Decision::Reply("fallback: anything".into()));
    }

    #[test]
    fn test_threshold_clamping() {
        let router = Router::new(fallback).threshold(1.5);
        assert_eq!(router.threshold, 1.0);

        let router = Router::new(fallback).threshold(-0.5);
        assert_eq!(router.threshold, 0.0);
    }
}
