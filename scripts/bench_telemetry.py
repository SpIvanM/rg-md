#!/usr/bin/env python3
"""Telemetry analysis script for rg-md A/B benchmark evaluation."""

import json
import sys
from pathlib import Path

def analyze_benchmarks(json_path: Path):
    if not json_path.exists():
        print(f"File not found: {json_path}")
        return

    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    print("=" * 90)
    print(f"{'Variant':24} | {'Score':5} | {'Final Ctx':9} | {'Turns':5} | {'Time (s)':8} | {'Cost ($)':8}")
    print("-" * 90)

    for item in data:
        vid = item.get("variant_id", "unknown")
        score = item.get("plan_score", 0)
        ctx = item.get("final_context_tokens", 0)
        turns = item.get("task_turns", 0)
        time_s = item.get("task_time_s", 0.0)
        cost = item.get("task_cost", 0.0)
        print(f"{vid:24} | {score:5} | {ctx:9,d} | {turns:5} | {time_s:8.2f} | ${cost:7.4f}")
    print("=" * 90)

if __name__ == "__main__":
    default_path = Path("/home/ivanm/projects/ast-tools-ab-test/docs/results/benchmarks/benchmark_results_cycle3.json")
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else default_path
    analyze_benchmarks(target)
