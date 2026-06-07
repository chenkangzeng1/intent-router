# intent-router

Composable intent routing for Rust. Cascade from regex → embedding → local LLM → remote LLM with confidence-based short-circuiting. Zero business intrusion: you define logic, we orchestrate. Built for AI agent pipelines where latency and cost matter.

## Quick Start

```rust
use intent_router::{Handler, HandlerResult, Router, RoutingStrategy};

#[derive(Clone, PartialEq, Debug)]
enum Decision { Execute(String), Reply(String) }

struct MyHandler;
impl Handler<String, Decision, ()> for MyHandler {
    fn handle(&self, input: &String, _ctx: &()) -> HandlerResult<Decision> {
        if input.contains("install") {
            HandlerResult::new(Decision::Execute("install".into()), 0.95)
        } else {
            HandlerResult::new(Decision::Reply(input.clone()), 0.0)
        }
    }
}

let router = Router::new(|input: &String, _| Decision::Reply(input.clone()))
    .with(MyHandler);

let result = router.route(&"install docker".into(), &());
assert_eq!(result, Decision::Execute("install".into()));
```

## Design Principles

- **Framework provides skeleton, user fills the meat**: You implement `Handler`, we handle orchestration
- **Performance-first, short-circuit by default**: First handler with `confidence >= threshold` wins
- **Zero business intrusion**: No predefined intent enums — your types, your logic
- **Errors are low confidence**: Handler failures naturally cascade to next layer

## Routing Strategies

| Strategy | Behavior | Use Case |
|----------|----------|----------|
| `FirstMatch` | Short-circuit on first hit | Performance-first (default) |
| `HighestScore` | Run all, pick highest confidence | Accuracy-first |
| `WeightedVote` | Weighted voting across handlers | Multi-model fusion |

## License

MIT
