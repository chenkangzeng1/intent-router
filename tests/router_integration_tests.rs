use intent_router::{Handler, HandlerResult, Router, RoutingStrategy};

#[derive(Debug, Clone, PartialEq)]
enum Decision {
    Execute(String),
    Reply(String),
}

struct KeywordHandler {
    keyword: String,
    decision: Decision,
    confidence: f32,
}

impl Handler<String, Decision, ()> for KeywordHandler {
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
fn test_basic_routing() {
    let router = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"我要安装软件".into(), &()),
        Decision::Execute("install".into())
    );
}

#[test]
fn test_no_match_fallback() {
    let router = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"hello world".into(), &()),
        Decision::Reply("fallback: hello world".into())
    );
}

#[test]
fn test_short_circuit_first_match() {
    let router = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.9,
        })
        .with(KeywordHandler {
            keyword: "docker".into(),
            decision: Decision::Execute("docker".into()),
            confidence: 0.8,
        });

    // 两个都匹配，但第一个命中即返回
    assert_eq!(
        router.route(&"安装 docker".into(), &()),
        Decision::Execute("install".into())
    );
}

#[test]
fn test_threshold_blocks_low_confidence() {
    let router = Router::new(fallback)
        .threshold(0.8)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.7,
        });

    assert_eq!(
        router.route(&"安装软件".into(), &()),
        Decision::Reply("fallback: 安装软件".into())
    );
}

#[test]
fn test_threshold_allows_high_confidence() {
    let router = Router::new(fallback)
        .threshold(0.8)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"安装软件".into(), &()),
        Decision::Execute("install".into())
    );
}

#[test]
fn test_zero_confidence_skips_to_next() {
    let router = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.0,
        })
        .with(KeywordHandler {
            keyword: "docker".into(),
            decision: Decision::Execute("docker".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"安装 docker".into(), &()),
        Decision::Execute("docker".into())
    );
}

#[test]
fn test_highest_score_strategy() {
    let router = Router::new(fallback)
        .strategy(RoutingStrategy::HighestScore)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.6,
        })
        .with(KeywordHandler {
            keyword: "docker".into(),
            decision: Decision::Execute("docker".into()),
            confidence: 0.9,
        });

    // HighestScore 会选 confidence 最高的
    assert_eq!(
        router.route(&"安装 docker".into(), &()),
        Decision::Execute("docker".into())
    );
}

#[test]
fn test_highest_score_with_threshold_filter() {
    let router = Router::new(fallback)
        .strategy(RoutingStrategy::HighestScore)
        .threshold(0.7)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.6, // 低于阈值，不参与
        })
        .with(KeywordHandler {
            keyword: "docker".into(),
            decision: Decision::Execute("docker".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"安装 docker".into(), &()),
        Decision::Execute("docker".into())
    );
}

#[test]
fn test_highest_score_all_below_threshold_fallback() {
    let router = Router::new(fallback)
        .strategy(RoutingStrategy::HighestScore)
        .threshold(0.9)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.6,
        })
        .with(KeywordHandler {
            keyword: "docker".into(),
            decision: Decision::Execute("docker".into()),
            confidence: 0.8,
        });

    assert_eq!(
        router.route(&"安装 docker".into(), &()),
        Decision::Reply("fallback: 安装 docker".into())
    );
}

#[test]
fn test_weighted_vote_strategy_fallback_to_highest() {
    let router = Router::new(fallback)
        .strategy(RoutingStrategy::WeightedVote)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("install".into()),
            confidence: 0.6,
        })
        .with(KeywordHandler {
            keyword: "docker".into(),
            decision: Decision::Execute("docker".into()),
            confidence: 0.9,
        });

    // WeightedVote 默认实现 fallback 到 HighestScore
    assert_eq!(
        router.route(&"安装 docker".into(), &()),
        Decision::Execute("docker".into())
    );
}

#[test]
fn test_empty_router() {
    let router: Router<String, Decision, ()> = Router::new(fallback);
    assert_eq!(
        router.route(&"anything".into(), &()),
        Decision::Reply("fallback: anything".into())
    );
}

#[test]
fn test_multiple_handlers_all_miss() {
    let router = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "aaa".into(),
            decision: Decision::Execute("aaa".into()),
            confidence: 0.9,
        })
        .with(KeywordHandler {
            keyword: "bbb".into(),
            decision: Decision::Execute("bbb".into()),
            confidence: 0.9,
        })
        .with(KeywordHandler {
            keyword: "ccc".into(),
            decision: Decision::Execute("ccc".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"xyz".into(), &()),
        Decision::Reply("fallback: xyz".into())
    );
}

#[test]
fn test_handler_order_matters() {
    let router1 = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("first".into()),
            confidence: 0.9,
        })
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("second".into()),
            confidence: 0.95,
        });

    let router2 = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("second".into()),
            confidence: 0.95,
        })
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("first".into()),
            confidence: 0.9,
        });

    // FirstMatch 策略下，顺序决定结果
    assert_eq!(
        router1.route(&"安装".into(), &()),
        Decision::Execute("first".into())
    );
    assert_eq!(
        router2.route(&"安装".into(), &()),
        Decision::Execute("second".into())
    );
}

#[test]
fn test_context_passed_to_handler() {
    struct ContextAwareHandler;

    impl Handler<String, Decision, String> for ContextAwareHandler {
        fn handle(&self, input: &String, ctx: &String) -> HandlerResult<Decision> {
            if ctx == "admin" && input.contains("删除") {
                HandlerResult::new(Decision::Execute("delete".into()), 1.0)
            } else {
                HandlerResult::new(Decision::Reply("forbidden".into()), 0.0)
            }
        }
    }

    let router = Router::new(|_: &String, _: &String| Decision::Reply("fallback".into()))
        .with(ContextAwareHandler);

    assert_eq!(
        router.route(&"删除数据".into(), &"admin".into()),
        Decision::Execute("delete".into())
    );
    assert_eq!(
        router.route(&"删除数据".into(), &"guest".into()),
        Decision::Reply("fallback".into())
    );
}

#[test]
fn test_builder_pattern_chaining() {
    let router = Router::new(fallback)
        .threshold(0.5)
        .strategy(RoutingStrategy::FirstMatch)
        .with(KeywordHandler {
            keyword: "test".into(),
            decision: Decision::Execute("test".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"test".into(), &()),
        Decision::Execute("test".into())
    );
}

#[test]
fn test_confidence_boundary_values() {
    let router = Router::new(fallback)
        .threshold(0.5)
        .with(KeywordHandler {
            keyword: "exact".into(),
            decision: Decision::Execute("exact".into()),
            confidence: 0.5, // 刚好等于阈值
        });

    assert_eq!(
        router.route(&"exact".into(), &()),
        Decision::Execute("exact".into())
    );
}

#[test]
fn test_exactly_below_threshold() {
    let router = Router::new(fallback)
        .threshold(0.5)
        .with(KeywordHandler {
            keyword: "below".into(),
            decision: Decision::Execute("below".into()),
            confidence: 0.49, // 刚好低于阈值
        });

    assert_eq!(
        router.route(&"below".into(), &()),
        Decision::Reply("fallback: below".into())
    );
}

#[test]
fn test_complex_cascading_scenario() {
    // 模拟真实场景：规则 -> 向量 -> 本地模型 -> 远程模型
    let router = Router::new(fallback)
        .with(KeywordHandler {
            keyword: "安装".into(),
            decision: Decision::Execute("rule_install".into()),
            confidence: 1.0,
        })
        .with(KeywordHandler {
            keyword: "相似".into(),
            decision: Decision::Execute("embedding_similar".into()),
            confidence: 0.85,
        })
        .with(KeywordHandler {
            keyword: "复杂".into(),
            decision: Decision::Execute("local_complex".into()),
            confidence: 0.75,
        })
        .with(KeywordHandler {
            keyword: "推理".into(),
            decision: Decision::Execute("remote_reasoning".into()),
            confidence: 0.9,
        });

    assert_eq!(
        router.route(&"安装软件".into(), &()),
        Decision::Execute("rule_install".into())
    );
    assert_eq!(
        router.route(&"相似度计算".into(), &()),
        Decision::Execute("embedding_similar".into())
    );
    assert_eq!(
        router.route(&"复杂任务".into(), &()),
        Decision::Execute("local_complex".into())
    );
    assert_eq!(
        router.route(&"深度推理".into(), &()),
        Decision::Execute("remote_reasoning".into())
    );
    assert_eq!(
        router.route(&"未知请求".into(), &()),
        Decision::Reply("fallback: 未知请求".into())
    );
}
