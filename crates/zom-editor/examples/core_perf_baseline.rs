//! 编辑引擎核心能力性能基准（M3）。
//!
//! 用法：
//! `cargo run --release -p zom-editor --example core_perf_baseline -- [--enforce]`

use std::{env, process, time::Instant};

use zom_editor::{EditorState, apply_editor_invocation, wrap_visual_line};
use zom_protocol::{EditorAction, EditorInvocation, Position};

#[derive(Debug, Clone, Copy)]
struct Thresholds {
    insert_1mb_p95_us: f64,
    insert_5mb_p95_us: f64,
    delete_1mb_p95_us: f64,
    move_right_1mb_p95_us: f64,
    mapping_5mb_p95_us: f64,
    wrap_1mb_ms: f64,
}

#[derive(Debug, Clone)]
struct OpMetric {
    name: &'static str,
    operations: usize,
    total_ms: f64,
    avg_us: f64,
    p95_us: f64,
}

impl OpMetric {
    fn throughput_ops_per_sec(&self) -> f64 {
        let total_secs = self.total_ms / 1000.0;
        if total_secs == 0.0 {
            return 0.0;
        }
        self.operations as f64 / total_secs
    }
}

fn main() {
    let enforce = env::args().skip(1).any(|arg| arg == "--enforce");
    let thresholds = Thresholds {
        insert_1mb_p95_us: 1500.0,
        insert_5mb_p95_us: 3000.0,
        delete_1mb_p95_us: 1500.0,
        move_right_1mb_p95_us: 600.0,
        mapping_5mb_p95_us: 200.0,
        wrap_1mb_ms: 350.0,
    };

    let text_1mb = generate_ascii_text(1 * 1024 * 1024);
    let text_5mb = generate_ascii_text(5 * 1024 * 1024);
    let line_1mb = "a".repeat(1 * 1024 * 1024);

    let insert_1mb = measure_editor_op_metric(
        "insert_tail_1mb",
        &text_1mb,
        300,
        |state, cursor| apply_editor_invocation(state, cursor, &EditorInvocation::insert_text("x")),
    );
    let insert_5mb = measure_editor_op_metric(
        "insert_tail_5mb",
        &text_5mb,
        300,
        |state, cursor| apply_editor_invocation(state, cursor, &EditorInvocation::insert_text("x")),
    );
    let delete_1mb = measure_editor_op_metric("delete_backward_tail_1mb", &text_1mb, 300, |state, cursor| {
        apply_editor_invocation(
            state,
            cursor,
            &EditorInvocation::from(EditorAction::DeleteBackward),
        )
    });
    let move_right_1mb = measure_editor_op_metric("move_right_1mb", &text_1mb, 20_000, |state, cursor| {
        apply_editor_invocation(state, cursor, &EditorInvocation::from(EditorAction::MoveRight))
    });
    let mapping_5mb = measure_mapping_metric(&text_5mb);
    let wrap_1mb_ms = measure_wrap_time_ms(&line_1mb, 120);

    println!("=== Zom Editor Core Performance Baseline ===");
    println!(
        "{:<30} {:>12} {:>12} {:>12}",
        "scenario", "avg(us)", "p95(us)", "ops/s"
    );
    for metric in [
        &insert_1mb,
        &insert_5mb,
        &delete_1mb,
        &move_right_1mb,
        &mapping_5mb,
    ] {
        println!(
            "{:<30} {:>12.2} {:>12.2} {:>12.2}",
            metric.name,
            metric.avg_us,
            metric.p95_us,
            metric.throughput_ops_per_sec()
        );
    }
    println!("{:<30} {:>12.2}", "wrap_visual_line_1mb(ms)", wrap_1mb_ms);

    let regressions = collect_regressions(
        &thresholds,
        &insert_1mb,
        &insert_5mb,
        &delete_1mb,
        &move_right_1mb,
        &mapping_5mb,
        wrap_1mb_ms,
    );

    if regressions.is_empty() {
        println!("baseline-check: PASS");
        return;
    }

    println!("baseline-check: WARN");
    for regression in &regressions {
        println!(" - {regression}");
    }

    if enforce {
        eprintln!("baseline-check: FAIL (--enforce)");
        process::exit(1);
    }
}

fn generate_ascii_text(target_bytes: usize) -> String {
    let line = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ\n";
    let mut out = String::with_capacity(target_bytes + line.len());
    while out.len() < target_bytes {
        out.push_str(line);
    }
    out.truncate(target_bytes);
    out
}

fn measure_editor_op_metric(
    name: &'static str,
    initial_text: &str,
    operations: usize,
    mut op: impl FnMut(&EditorState, Position) -> zom_editor::InvocationResult,
) -> OpMetric {
    let mut state = EditorState::from_text(initial_text);
    let mut cursor = state.offset_to_position(state.len());

    let mut costs = Vec::with_capacity(operations);
    let total_start = Instant::now();
    for _ in 0..operations {
        let started = Instant::now();
        let result = op(&state, cursor);
        costs.push(started.elapsed().as_nanos());
        state = result.state;
        cursor = result.cursor;
    }
    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    let p95_us = p95_nanos(&mut costs) as f64 / 1000.0;
    let avg_us = (costs.iter().sum::<u128>() as f64 / operations as f64) / 1000.0;

    OpMetric {
        name,
        operations,
        total_ms,
        avg_us,
        p95_us,
    }
}

fn measure_mapping_metric(initial_text: &str) -> OpMetric {
    let state = EditorState::from_text(initial_text);
    let mut offsets = Vec::new();
    let step = 256usize;
    let mut cursor = 0usize;
    while cursor < state.len() {
        offsets.push(cursor);
        cursor = cursor.saturating_add(step);
    }
    offsets.push(state.len());
    let operations = offsets.len();

    let mut costs = Vec::with_capacity(operations);
    let total_start = Instant::now();
    for offset in offsets {
        let started = Instant::now();
        let position = state.offset_to_position(offset);
        let mapped_offset = state.position_to_offset(position);
        assert!(mapped_offset <= state.len(), "mapped offset should be in bounds");
        costs.push(started.elapsed().as_nanos());
    }
    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    let p95_us = p95_nanos(&mut costs) as f64 / 1000.0;
    let avg_us = (costs.iter().sum::<u128>() as f64 / operations as f64) / 1000.0;

    OpMetric {
        name: "offset_position_roundtrip_5mb",
        operations,
        total_ms,
        avg_us,
        p95_us,
    }
}

fn measure_wrap_time_ms(line: &str, width: usize) -> f64 {
    let started = Instant::now();
    let segments = wrap_visual_line(line, width);
    assert!(!segments.is_empty(), "wrap output should not be empty");
    started.elapsed().as_secs_f64() * 1000.0
}

fn p95_nanos(costs: &mut [u128]) -> u128 {
    costs.sort_unstable();
    if costs.is_empty() {
        return 0;
    }
    let rank = ((costs.len() as f64) * 0.95).ceil() as usize;
    let index = rank.saturating_sub(1).min(costs.len() - 1);
    costs[index]
}

fn collect_regressions(
    thresholds: &Thresholds,
    insert_1mb: &OpMetric,
    insert_5mb: &OpMetric,
    delete_1mb: &OpMetric,
    move_right_1mb: &OpMetric,
    mapping_5mb: &OpMetric,
    wrap_1mb_ms: f64,
) -> Vec<String> {
    let mut out = Vec::new();

    check_p95(
        &mut out,
        insert_1mb.name,
        insert_1mb.p95_us,
        thresholds.insert_1mb_p95_us,
    );
    check_p95(
        &mut out,
        insert_5mb.name,
        insert_5mb.p95_us,
        thresholds.insert_5mb_p95_us,
    );
    check_p95(
        &mut out,
        delete_1mb.name,
        delete_1mb.p95_us,
        thresholds.delete_1mb_p95_us,
    );
    check_p95(
        &mut out,
        move_right_1mb.name,
        move_right_1mb.p95_us,
        thresholds.move_right_1mb_p95_us,
    );
    check_p95(
        &mut out,
        mapping_5mb.name,
        mapping_5mb.p95_us,
        thresholds.mapping_5mb_p95_us,
    );
    if wrap_1mb_ms > thresholds.wrap_1mb_ms {
        out.push(format!(
            "wrap_visual_line_1mb(ms): {:.2} > {:.2}",
            wrap_1mb_ms, thresholds.wrap_1mb_ms
        ));
    }

    out
}

fn check_p95(regressions: &mut Vec<String>, name: &str, actual: f64, threshold: f64) {
    if actual > threshold {
        regressions.push(format!("{name}: p95 {:.2}us > {:.2}us", actual, threshold));
    }
}
