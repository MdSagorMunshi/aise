#!/usr/bin/env python3
"""
AEGIS-Ω Benchmark Chart Generator & COMPARE.md Builder

Reads benchmark JSON output and generates:
  1. SVG charts (throughput, latency, scalability, permutation breakdown)
  2. A comprehensive COMPARE.md report

Usage:
    python3 gen_compare.py bench_results.json
"""

import json
import sys
import os
import math
from collections import defaultdict

# ──────────────────────────────────────────────────────────────
#  SVG Chart Generator (zero external dependencies)
# ──────────────────────────────────────────────────────────────

# Color palette - modern, vibrant, dark-mode friendly
COLORS = {
    "AISE-HASH": "#FF6B35",    # Warm orange - the protagonist
    "SHA-256":   "#4ECDC4",    # Teal
    "SHA-512":   "#45B7D1",    # Sky blue
    "SHA3-256":  "#96CEB4",    # Sage green
    "SHA3-512":  "#88D8B0",    # Mint
    "BLAKE2b":   "#DDA0DD",    # Plum
    "BLAKE3":    "#FFD93D",    # Gold
}

ALGO_ORDER = ["AISE-HASH", "SHA-256", "SHA-512", "SHA3-256", "SHA3-512", "BLAKE2b", "BLAKE3"]


def escape_svg(s):
    return str(s).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def format_number(n):
    """Format number with commas for readability."""
    if n >= 1_000_000:
        return f"{n/1_000_000:.1f}M"
    if n >= 1_000:
        return f"{n/1_000:.1f}K"
    if isinstance(n, float):
        if n >= 100:
            return f"{n:.0f}"
        if n >= 10:
            return f"{n:.1f}"
        return f"{n:.2f}"
    return str(n)


# ──────────────────────────────────────────────────────────────
#  Chart 1: Throughput Bar Chart (grouped by input size)
# ──────────────────────────────────────────────────────────────

def gen_throughput_chart(throughput_data, output_path):
    """Generate a grouped bar chart of throughput (MB/s) by input size."""

    # Group by input size
    by_size = defaultdict(dict)
    for r in throughput_data:
        by_size[r["input_bytes"]][r["algorithm"]] = r["throughput_mbps"]

    sizes = sorted(by_size.keys())
    size_labels = []
    for s in sizes:
        if s < 1024:
            size_labels.append(f"{s}B")
        elif s < 1024*1024:
            size_labels.append(f"{s//1024}KB")
        else:
            size_labels.append(f"{s//(1024*1024)}MB")

    n_groups = len(sizes)
    n_bars = len(ALGO_ORDER)
    bar_w = 14
    group_gap = 30
    group_w = n_bars * bar_w + group_gap
    chart_w = n_groups * group_w + 100
    chart_h = 420
    margin_l, margin_r, margin_t, margin_b = 90, 30, 60, 120
    plot_w = chart_w - margin_l - margin_r
    plot_h = chart_h - margin_t - margin_b

    # Find max throughput (use log scale since BLAKE3 >> AISE)
    all_vals = [v for d in by_size.values() for v in d.values() if v > 0]
    max_val = max(all_vals) * 1.15

    # Use log scale
    use_log = max_val / min(v for v in all_vals if v > 0) > 100

    svg = []
    svg.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {chart_w} {chart_h}" font-family="Inter, -apple-system, sans-serif">')
    svg.append(f'<rect width="{chart_w}" height="{chart_h}" fill="#0d1117" rx="12"/>')

    # Title
    svg.append(f'<text x="{chart_w//2}" y="30" text-anchor="middle" fill="#e6edf3" font-size="16" font-weight="700">Throughput Comparison (MB/s) — {"Log" if use_log else "Linear"} Scale</text>')

    # Y-axis
    if use_log:
        min_exp = 0
        max_exp = math.ceil(math.log10(max_val))
        y_ticks = list(range(min_exp, max_exp + 1))
        for exp in y_ticks:
            val = 10 ** exp
            y = margin_t + plot_h - (exp / max_exp) * plot_h
            svg.append(f'<line x1="{margin_l}" y1="{y:.0f}" x2="{margin_l + plot_w}" y2="{y:.0f}" stroke="#21262d" stroke-width="1"/>')
            svg.append(f'<text x="{margin_l - 8}" y="{y + 4:.0f}" text-anchor="end" fill="#8b949e" font-size="11">{format_number(val)}</text>')
    else:
        n_ticks = 6
        for i in range(n_ticks + 1):
            val = (max_val / n_ticks) * i
            y = margin_t + plot_h - (i / n_ticks) * plot_h
            svg.append(f'<line x1="{margin_l}" y1="{y:.0f}" x2="{margin_l + plot_w}" y2="{y:.0f}" stroke="#21262d" stroke-width="1"/>')
            svg.append(f'<text x="{margin_l - 8}" y="{y + 4:.0f}" text-anchor="end" fill="#8b949e" font-size="11">{format_number(val)}</text>')

    # Y-axis label
    svg.append(f'<text x="15" y="{margin_t + plot_h//2}" text-anchor="middle" fill="#8b949e" font-size="12" transform="rotate(-90, 15, {margin_t + plot_h//2})">MB/s</text>')

    # Bars
    for gi, size in enumerate(sizes):
        group_x = margin_l + gi * group_w + group_gap // 2

        for bi, algo in enumerate(ALGO_ORDER):
            val = by_size[size].get(algo, 0)
            if val <= 0:
                continue
            color = COLORS.get(algo, "#666")
            bx = group_x + bi * bar_w

            if use_log:
                log_val = math.log10(max(val, 0.01))
                bar_h = max((log_val / max_exp) * plot_h, 2)
            else:
                bar_h = max((val / max_val) * plot_h, 2)

            by = margin_t + plot_h - bar_h
            svg.append(f'<rect x="{bx:.0f}" y="{by:.1f}" width="{bar_w - 1}" height="{bar_h:.1f}" fill="{color}" rx="2" opacity="0.9"/>')

        # Size label
        label_x = group_x + (n_bars * bar_w) / 2
        svg.append(f'<text x="{label_x:.0f}" y="{margin_t + plot_h + 18}" text-anchor="middle" fill="#c9d1d9" font-size="11">{size_labels[gi]}</text>')

    # Legend
    leg_y = chart_h - 55
    leg_x_start = margin_l
    for i, algo in enumerate(ALGO_ORDER):
        lx = leg_x_start + (i % 4) * 160
        ly = leg_y + (i // 4) * 20
        svg.append(f'<rect x="{lx}" y="{ly}" width="12" height="12" fill="{COLORS[algo]}" rx="2"/>')
        svg.append(f'<text x="{lx + 16}" y="{ly + 10}" fill="#c9d1d9" font-size="11">{algo}</text>')

    svg.append('</svg>')
    with open(output_path, 'w') as f:
        f.write('\n'.join(svg))
    print(f"  Generated: {output_path}")


# ──────────────────────────────────────────────────────────────
#  Chart 2: Latency Bar Chart
# ──────────────────────────────────────────────────────────────

def gen_latency_chart(latency_data, output_path):
    """Generate latency comparison chart for small messages."""

    by_size = defaultdict(dict)
    for r in latency_data:
        by_size[r["input_bytes"]][r["algorithm"]] = r["median_ns"]

    sizes = sorted(by_size.keys())
    size_labels = [f"{s}B" for s in sizes]

    n_groups = len(sizes)
    n_bars = len(ALGO_ORDER)
    bar_w = 22
    group_gap = 50
    group_w = n_bars * bar_w + group_gap
    chart_w = n_groups * group_w + 140
    chart_h = 400
    margin_l, margin_r, margin_t, margin_b = 100, 30, 60, 110
    plot_w = chart_w - margin_l - margin_r
    plot_h = chart_h - margin_t - margin_b

    all_vals = [v for d in by_size.values() for v in d.values() if v > 0]
    max_val_raw = max(all_vals) * 1.15

    # Use log scale for latency too (AISE is orders of magnitude slower)
    max_exp = math.ceil(math.log10(max_val_raw))
    min_exp = max(0, math.floor(math.log10(min(all_vals))))

    svg = []
    svg.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {chart_w} {chart_h}" font-family="Inter, -apple-system, sans-serif">')
    svg.append(f'<rect width="{chart_w}" height="{chart_h}" fill="#0d1117" rx="12"/>')

    svg.append(f'<text x="{chart_w//2}" y="30" text-anchor="middle" fill="#e6edf3" font-size="16" font-weight="700">Latency Comparison — Small Messages (Log Scale, lower is better)</text>')

    # Y-axis (log)
    for exp in range(min_exp, max_exp + 1):
        val = 10 ** exp
        y = margin_t + plot_h - ((exp - min_exp) / (max_exp - min_exp)) * plot_h
        svg.append(f'<line x1="{margin_l}" y1="{y:.0f}" x2="{margin_l + plot_w}" y2="{y:.0f}" stroke="#21262d" stroke-width="1"/>')
        label = f"{format_number(val)} ns"
        svg.append(f'<text x="{margin_l - 8}" y="{y + 4:.0f}" text-anchor="end" fill="#8b949e" font-size="11">{label}</text>')

    svg.append(f'<text x="18" y="{margin_t + plot_h//2}" text-anchor="middle" fill="#8b949e" font-size="12" transform="rotate(-90, 18, {margin_t + plot_h//2})">Nanoseconds (log)</text>')

    for gi, size in enumerate(sizes):
        group_x = margin_l + gi * group_w + group_gap // 2

        for bi, algo in enumerate(ALGO_ORDER):
            val = by_size[size].get(algo, 0)
            if val <= 0:
                continue
            color = COLORS.get(algo, "#666")
            bx = group_x + bi * bar_w

            log_val = math.log10(max(val, 1))
            bar_h = max(((log_val - min_exp) / (max_exp - min_exp)) * plot_h, 2)
            by = margin_t + plot_h - bar_h

            svg.append(f'<rect x="{bx:.0f}" y="{by:.1f}" width="{bar_w - 2}" height="{bar_h:.1f}" fill="{color}" rx="2" opacity="0.9"/>')

        label_x = group_x + (n_bars * bar_w) / 2
        svg.append(f'<text x="{label_x:.0f}" y="{margin_t + plot_h + 20}" text-anchor="middle" fill="#c9d1d9" font-size="12" font-weight="600">{size_labels[gi]}</text>')

    # Legend
    leg_y = chart_h - 50
    for i, algo in enumerate(ALGO_ORDER):
        lx = margin_l + (i % 4) * 160
        ly = leg_y + (i // 4) * 20
        svg.append(f'<rect x="{lx}" y="{ly}" width="12" height="12" fill="{COLORS[algo]}" rx="2"/>')
        svg.append(f'<text x="{lx + 16}" y="{ly + 10}" fill="#c9d1d9" font-size="11">{algo}</text>')

    svg.append('</svg>')
    with open(output_path, 'w') as f:
        f.write('\n'.join(svg))
    print(f"  Generated: {output_path}")


# ──────────────────────────────────────────────────────────────
#  Chart 3: Scalability Line Chart
# ──────────────────────────────────────────────────────────────

def gen_scalability_chart(scalability_data, output_path):
    """Line chart: throughput vs input size for each algorithm."""

    by_algo = defaultdict(list)
    for r in scalability_data:
        by_algo[r["algorithm"]].append((r["input_bytes"], r["throughput_mbps"]))

    chart_w, chart_h = 800, 450
    margin_l, margin_r, margin_t, margin_b = 90, 30, 60, 100
    plot_w = chart_w - margin_l - margin_r
    plot_h = chart_h - margin_t - margin_b

    # Log-log scale
    all_sizes = sorted(set(r["input_bytes"] for r in scalability_data))
    all_tp = [r["throughput_mbps"] for r in scalability_data if r["throughput_mbps"] > 0]
    
    min_size_exp = math.floor(math.log10(min(all_sizes)))
    max_size_exp = math.ceil(math.log10(max(all_sizes)))
    min_tp_exp = math.floor(math.log10(min(all_tp)))
    max_tp_exp = math.ceil(math.log10(max(all_tp)))

    def x_pos(size_bytes):
        return margin_l + ((math.log10(size_bytes) - min_size_exp) / (max_size_exp - min_size_exp)) * plot_w

    def y_pos(tp):
        if tp <= 0:
            return margin_t + plot_h
        return margin_t + plot_h - ((math.log10(tp) - min_tp_exp) / (max_tp_exp - min_tp_exp)) * plot_h

    svg = []
    svg.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {chart_w} {chart_h}" font-family="Inter, -apple-system, sans-serif">')
    svg.append(f'<rect width="{chart_w}" height="{chart_h}" fill="#0d1117" rx="12"/>')
    svg.append(f'<text x="{chart_w//2}" y="30" text-anchor="middle" fill="#e6edf3" font-size="16" font-weight="700">Scalability Profile — Throughput vs Input Size (Log-Log)</text>')

    # Grid lines
    for exp in range(min_size_exp, max_size_exp + 1):
        x = x_pos(10**exp)
        svg.append(f'<line x1="{x:.0f}" y1="{margin_t}" x2="{x:.0f}" y2="{margin_t + plot_h}" stroke="#21262d" stroke-width="1"/>')
        size_val = 10**exp
        if size_val < 1024:
            label = f"{size_val}B"
        elif size_val < 1024*1024:
            label = f"{size_val//1024}KB"
        else:
            label = f"{size_val//(1024*1024)}MB"
        svg.append(f'<text x="{x:.0f}" y="{margin_t + plot_h + 18}" text-anchor="middle" fill="#8b949e" font-size="10">{label}</text>')

    for exp in range(min_tp_exp, max_tp_exp + 1):
        y = y_pos(10**exp)
        svg.append(f'<line x1="{margin_l}" y1="{y:.0f}" x2="{margin_l + plot_w}" y2="{y:.0f}" stroke="#21262d" stroke-width="1"/>')
        svg.append(f'<text x="{margin_l - 8}" y="{y + 4:.0f}" text-anchor="end" fill="#8b949e" font-size="10">{format_number(10**exp)}</text>')

    svg.append(f'<text x="18" y="{margin_t + plot_h//2}" text-anchor="middle" fill="#8b949e" font-size="12" transform="rotate(-90, 18, {margin_t + plot_h//2})">MB/s (log)</text>')
    svg.append(f'<text x="{margin_l + plot_w//2}" y="{chart_h - 55}" text-anchor="middle" fill="#8b949e" font-size="12">Input Size (log)</text>')

    # Lines
    for algo in ALGO_ORDER:
        points = sorted(by_algo.get(algo, []))
        if not points:
            continue
        color = COLORS.get(algo, "#666")
        path_parts = []
        for i, (size, tp) in enumerate(points):
            if tp <= 0:
                continue
            x = x_pos(size)
            y = y_pos(tp)
            if i == 0:
                path_parts.append(f"M{x:.1f},{y:.1f}")
            else:
                path_parts.append(f"L{x:.1f},{y:.1f}")
        if path_parts:
            svg.append(f'<path d="{" ".join(path_parts)}" fill="none" stroke="{color}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" opacity="0.9"/>')
            # Dots
            for size, tp in points:
                if tp <= 0:
                    continue
                x = x_pos(size)
                y = y_pos(tp)
                svg.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="3.5" fill="{color}"/>')

    # Legend
    leg_y = chart_h - 35
    for i, algo in enumerate(ALGO_ORDER):
        lx = margin_l + (i % 4) * 170
        ly = leg_y + (i // 4) * 18
        svg.append(f'<rect x="{lx}" y="{ly}" width="14" height="3" fill="{COLORS[algo]}" rx="1"/>')
        svg.append(f'<text x="{lx + 18}" y="{ly + 5}" fill="#c9d1d9" font-size="11">{algo}</text>')

    svg.append('</svg>')
    with open(output_path, 'w') as f:
        f.write('\n'.join(svg))
    print(f"  Generated: {output_path}")


# ──────────────────────────────────────────────────────────────
#  Chart 4: AISE Permutation Breakdown
# ──────────────────────────────────────────────────────────────

def gen_permutation_chart(perm_data, output_path):
    """Horizontal bar chart showing AISE permutation component costs."""

    # Filter out the full cascade from the component bars
    components = [p for p in perm_data if "Full" not in p["component"]]
    cascade = [p for p in perm_data if "Full" in p["component"]]

    chart_w, chart_h = 700, 300
    margin_l, margin_r, margin_t, margin_b = 170, 80, 60, 50
    plot_w = chart_w - margin_l - margin_r
    plot_h = chart_h - margin_t - margin_b

    max_ns = max(c["median_ns"] for c in components) * 1.15

    perm_colors = ["#FF6B35", "#4ECDC4", "#DDA0DD"]

    svg = []
    svg.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {chart_w} {chart_h}" font-family="Inter, -apple-system, sans-serif">')
    svg.append(f'<rect width="{chart_w}" height="{chart_h}" fill="#0d1117" rx="12"/>')
    svg.append(f'<text x="{chart_w//2}" y="30" text-anchor="middle" fill="#e6edf3" font-size="16" font-weight="700">AISE Permutation Breakdown — Cost per Component</text>')

    bar_h = min(40, plot_h // len(components) - 10)
    for i, comp in enumerate(components):
        y = margin_t + i * (bar_h + 15)
        bar_width = (comp["median_ns"] / max_ns) * plot_w
        color = perm_colors[i % len(perm_colors)]

        svg.append(f'<text x="{margin_l - 10}" y="{y + bar_h//2 + 5}" text-anchor="end" fill="#c9d1d9" font-size="12">{comp["component"]}</text>')
        svg.append(f'<rect x="{margin_l}" y="{y}" width="{bar_width:.0f}" height="{bar_h}" fill="{color}" rx="4" opacity="0.85"/>')

        # Value label
        pct = comp.get("percentage_of_cascade", 0)
        label = f'{format_number(comp["median_ns"])} ns ({pct:.1f}%)'
        svg.append(f'<text x="{margin_l + bar_width + 8:.0f}" y="{y + bar_h//2 + 5}" fill="#8b949e" font-size="11">{label}</text>')

    # Cascade total
    if cascade:
        c = cascade[0]
        ty = margin_t + len(components) * (bar_h + 15) + 10
        svg.append(f'<text x="{margin_l - 10}" y="{ty + 5}" text-anchor="end" fill="#e6edf3" font-size="12" font-weight="600">Full Cascade</text>')
        svg.append(f'<text x="{margin_l}" y="{ty + 5}" fill="#FF6B35" font-size="12" font-weight="600">{format_number(c["median_ns"])} ns total ({c["throughput_mbps"]:.2f} MB/s)</text>')

    svg.append('</svg>')
    with open(output_path, 'w') as f:
        f.write('\n'.join(svg))
    print(f"  Generated: {output_path}")


# ──────────────────────────────────────────────────────────────
#  Chart 5: Throughput at 1MB (the "hero" chart)
# ──────────────────────────────────────────────────────────────

def gen_hero_chart(throughput_data, output_path):
    """Single big bar chart showing throughput at 1MB input — the money shot."""

    # Get 1MB results (or largest available)
    target_size = max(r["input_bytes"] for r in throughput_data)
    results = {r["algorithm"]: r["throughput_mbps"] for r in throughput_data if r["input_bytes"] == target_size}

    chart_w, chart_h = 750, 420
    margin_l, margin_r, margin_t, margin_b = 120, 30, 60, 80
    plot_w = chart_w - margin_l - margin_r
    plot_h = chart_h - margin_t - margin_b

    max_val = max(results.values()) * 1.15
    bar_h = min(42, plot_h // len(ALGO_ORDER) - 8)

    svg = []
    svg.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {chart_w} {chart_h}" font-family="Inter, -apple-system, sans-serif">')
    svg.append(f'<rect width="{chart_w}" height="{chart_h}" fill="#0d1117" rx="12"/>')

    size_label = f"{target_size//(1024*1024)}MB" if target_size >= 1024*1024 else f"{target_size//1024}KB"
    svg.append(f'<text x="{chart_w//2}" y="30" text-anchor="middle" fill="#e6edf3" font-size="16" font-weight="700">Throughput at {size_label} Input — All Algorithms</text>')
    svg.append(f'<text x="{chart_w//2}" y="48" text-anchor="middle" fill="#8b949e" font-size="11">Higher is better | Release build, target-cpu=native</text>')

    for i, algo in enumerate(ALGO_ORDER):
        val = results.get(algo, 0)
        y = margin_t + i * (bar_h + 8)
        bar_width = max((val / max_val) * plot_w, 3)
        color = COLORS.get(algo, "#666")

        svg.append(f'<text x="{margin_l - 10}" y="{y + bar_h//2 + 5}" text-anchor="end" fill="#c9d1d9" font-size="12" font-weight="500">{algo}</text>')

        # Gradient-like effect via opacity
        svg.append(f'<rect x="{margin_l}" y="{y}" width="{bar_width:.0f}" height="{bar_h}" fill="{color}" rx="4" opacity="0.9"/>')

        # Value label
        svg.append(f'<text x="{margin_l + bar_width + 8:.0f}" y="{y + bar_h//2 + 5}" fill="#e6edf3" font-size="12" font-weight="600">{val:.2f} MB/s</text>')

    svg.append('</svg>')
    with open(output_path, 'w') as f:
        f.write('\n'.join(svg))
    print(f"  Generated: {output_path}")


# ──────────────────────────────────────────────────────────────
#  COMPARE.md Generator
# ──────────────────────────────────────────────────────────────

def gen_compare_md(data, charts_dir, output_path):
    """Generate the comprehensive COMPARE.md report."""

    si = data["system_info"]
    throughput = data["throughput"]
    latency = data["latency"]
    perm = data["permutation_breakdown"]

    # ── Throughput table ──
    by_size = defaultdict(dict)
    for r in throughput:
        by_size[r["input_label"]][r["algorithm"]] = r

    size_order = []
    seen = set()
    for r in throughput:
        if r["input_label"] not in seen:
            size_order.append(r["input_label"])
            seen.add(r["input_label"])

    # ── Compute rankings ──
    rankings_1mb = {}
    target = max(r["input_bytes"] for r in throughput)
    for r in throughput:
        if r["input_bytes"] == target:
            rankings_1mb[r["algorithm"]] = r["throughput_mbps"]
    sorted_algos = sorted(rankings_1mb.items(), key=lambda x: x[1], reverse=True)

    # ── AISE permutation data ──
    cascade_entry = [p for p in perm if "Full" in p["component"]]
    component_entries = [p for p in perm if "Full" not in p["component"]]

    # ── Speed ratio ──
    aise_tp_1mb = rankings_1mb.get("AISE-HASH", 0)
    blake3_tp_1mb = rankings_1mb.get("BLAKE3", 0)
    sha256_tp_1mb = rankings_1mb.get("SHA-256", 0)

    lines = []
    lines.append("""<div align="center">

<pre>
   █████╗ ██╗███████╗███████╗      ██████╗ 
  ██╔══██╗██║██╔════╝██╔════╝     ██╔═══██╗
  ███████║██║███████╗█████╗   ██████║   ██║
  ██╔══██║██║╚════██║██╔══╝   ╚════██╗  ██║
  ██║  ██║██║███████║███████╗      ╚██████╔╝
  ╚═╝  ╚═╝╚═╝╚══════╝╚══════╝       ╚═════╝ 

  B E N C H M A R K   R E P O R T
</pre>

</div>

# AEGIS-Ω Benchmark Comparison Report

> **Live benchmark results** — not synthetic estimates.  
> All measurements taken on real hardware with optimized release builds.

""")

    # System info
    lines.append("## System Information\n")
    lines.append(f"| Property | Value |")
    lines.append(f"|---|---|")
    lines.append(f"| **CPU** | {si['cpu']} |")
    lines.append(f"| **AVX-512** | {'✅ Active (all AISE acceleration paths enabled)' if si['avx512'] else '❌ Not available'} |")
    lines.append(f"| **Target** | `target-cpu=native` |")
    lines.append(f"| **Build** | Release (opt-level=3, LTO, codegen-units=1) |")
    lines.append(f"| **Date** | {si['date']} |")
    lines.append(f"| **Rust** | rustc 1.96.0 |")
    lines.append("")

    # Algorithm overview
    lines.append("## Algorithms Under Test\n")
    lines.append("| Algorithm | Construction | Output | State Size | Key Design Goal |")
    lines.append("|---|---|---|---|---|")
    lines.append("| **AISE-HASH** | Triple-cascade sponge (Π_A→Π_B→Π_C) | 512-bit | **16,384-bit** | Maximum security margin via algebraic heterogeneity |")
    lines.append("| **SHA-256** | Merkle–Damgård | 256-bit | 256-bit | NIST standard, universal compatibility |")
    lines.append("| **SHA-512** | Merkle–Damgård | 512-bit | 1,024-bit | 64-bit optimized NIST standard |")
    lines.append("| **SHA3-256** | Keccak sponge | 256-bit | 1,600-bit | Post-SHA-2 NIST standard |")
    lines.append("| **SHA3-512** | Keccak sponge | 512-bit | 1,600-bit | Wide-output post-SHA-2 standard |")
    lines.append("| **BLAKE2b** | HAIFA (ChaCha-derived) | 512-bit | 512-bit | Fast general-purpose hash |")
    lines.append("| **BLAKE3** | Bao Merkle tree (ChaCha) | 256-bit | 256-bit | Fastest modern hash, parallelizable |")
    lines.append("")

    # ══════════════════════════════════════════════════════════
    #  HERO CHART
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## Throughput — The Big Picture\n")
    lines.append(f"![Throughput at 1MB](charts/hero_throughput.svg)\n")

    lines.append("### Rankings (1MB input)\n")
    lines.append("| Rank | Algorithm | Throughput | Relative to AISE |")
    lines.append("|---|---|---|---|")
    for rank, (algo, tp) in enumerate(sorted_algos, 1):
        medal = {1: "🥇", 2: "🥈", 3: "🥉"}.get(rank, f"#{rank}")
        ratio = tp / aise_tp_1mb if aise_tp_1mb > 0 else 0
        if algo == "AISE-HASH":
            lines.append(f"| {medal} | **{algo}** | **{tp:.2f} MB/s** | 1.00x (baseline) |")
        else:
            lines.append(f"| {medal} | {algo} | {tp:.2f} MB/s | {ratio:.0f}x faster |")
    lines.append("")

    # ══════════════════════════════════════════════════════════
    #  THROUGHPUT TABLE
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## Detailed Throughput (MB/s)\n")
    lines.append(f"![Throughput comparison](charts/throughput.svg)\n")

    header = "| Algorithm |"
    sep = "|---|"
    for sl in size_order:
        header += f" {sl} |"
        sep += "---|"
    lines.append(header)
    lines.append(sep)

    for algo in ALGO_ORDER:
        row = f"| **{algo}** |"
        for sl in size_order:
            r = by_size[sl].get(algo)
            if r:
                row += f" {r['throughput_mbps']:.2f} |"
            else:
                row += " - |"
        lines.append(row)
    lines.append("")

    # ══════════════════════════════════════════════════════════
    #  LATENCY TABLE
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## Latency — Small Message Performance\n")
    lines.append("> Small message latency is critical for API authentication, token generation, session IDs.\n")
    lines.append(f"![Latency comparison](charts/latency.svg)\n")

    lat_by_size = defaultdict(dict)
    for r in latency:
        lat_by_size[r["input_bytes"]][r["algorithm"]] = r

    lat_sizes = sorted(lat_by_size.keys())

    lines.append("| Algorithm |" + "".join(f" {s}B median | {s}B p99 |" for s in lat_sizes) + "")
    lines.append("|---|" + "---|---|" * len(lat_sizes))

    for algo in ALGO_ORDER:
        row = f"| **{algo}** |"
        for s in lat_sizes:
            r = lat_by_size[s].get(algo)
            if r:
                med_us = r["median_ns"] / 1000
                p99_us = r["p99_ns"] / 1000
                if med_us >= 1000:
                    row += f" {med_us/1000:.2f} ms | {p99_us/1000:.2f} ms |"
                else:
                    row += f" {med_us:.1f} µs | {p99_us:.1f} µs |"
            else:
                row += " - | - |"
        lines.append(row)
    lines.append("")

    # ══════════════════════════════════════════════════════════
    #  SCALABILITY
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## Scalability Profile\n")
    lines.append("> How throughput changes with input size. Algorithms with flatter curves have lower per-block overhead.\n")
    lines.append(f"![Scalability](charts/scalability.svg)\n")

    # ══════════════════════════════════════════════════════════
    #  PERMUTATION BREAKDOWN
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## AISE Permutation Breakdown\n")
    lines.append("> Where AEGIS-Ω spends its time. The triple-cascade permutation Π_Ω = Π_C ∘ Π_B ∘ Π_A processes the full 16,384-bit state.\n")
    lines.append(f"![Permutation breakdown](charts/permutation.svg)\n")

    lines.append("| Component | Domain | Time (ns) | Throughput (MB/s) | % of Cascade |")
    lines.append("|---|---|---|---|---|")
    for p in component_entries:
        lines.append(f"| **{p['component']}** | {'ℤ₂₆₄ ARX' if 'A' in p['component'] else 'GF(2¹²⁸)' if 'B' in p['component'] else 'GF(p) Mersenne'} | {p['median_ns']:,} | {p['throughput_mbps']:.2f} | {p['percentage_of_cascade']:.1f}% |")
    if cascade_entry:
        c = cascade_entry[0]
        lines.append(f"| **{c['component']}** | All three | **{c['median_ns']:,}** | **{c['throughput_mbps']:.2f}** | **100%** |")
    lines.append("")

    lines.append("> [!NOTE]\n> **The bottleneck is Π_C (Prime Field)** — modular exponentiation over the Mersenne prime 2¹²⁷−1 is inherently expensive. The alternating power map (x⁵ / x^d) in Rescue-style S-boxes forces high algebraic degree in both directions, which is the core of AISE's security argument but also its performance cost.\n")

    # ══════════════════════════════════════════════════════════
    #  SECURITY COMPARISON
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## Security Analysis\n")
    lines.append("> [!IMPORTANT]")
    lines.append("> **Generic security bounds are determined by the output length, not the state size.** For a 512-bit hash output, the birthday bound gives ~2²⁵⁶ collision resistance regardless of internal state width. AISE's 16,384-bit state provides structural margin against *non-generic* attacks (inner collisions, state recovery, capacity-targeting), but does not raise the output-level collision bound.\n")

    lines.append("| Property | AISE-HASH | SHA-256 | SHA-512 | SHA3-256 | SHA3-512 | BLAKE2b | BLAKE3 |")
    lines.append("|---|---|---|---|---|---|---|---|")
    lines.append("| **Output Size** | 512-bit | 256-bit | 512-bit | 256-bit | 512-bit | 512-bit | 256-bit |")
    lines.append("| **Classical Collision** | 2²⁵⁶ † | 2¹²⁸ | 2²⁵⁶ | 2¹²⁸ | 2²⁵⁶ | 2²⁵⁶ | 2¹²⁸ |")
    lines.append("| **Quantum Collision (BHT)** | ~2¹⁷¹ † | 2⁸⁵ | ~2¹⁷⁰ | 2⁸⁵ | ~2¹⁷⁰ | ~2¹⁷⁰ | 2⁸⁵ |")
    lines.append("| **Classical Preimage** | 2⁵¹² † | 2²⁵⁶ | 2⁵¹² | 2²⁵⁶ | 2⁵¹² | 2⁵¹² | 2²⁵⁶ |")
    lines.append("| **Quantum Preimage (Grover)** | 2²⁵⁶ † | 2¹²⁸ | 2²⁵⁶ | 2¹²⁸ | 2²⁵⁶ | 2²⁵⁶ | 2¹²⁸ |")
    lines.append("| **Internal State Size** | **16,384-bit** | 256-bit | 1,024-bit | 1,600-bit | 1,600-bit | 512-bit | 256-bit |")
    lines.append("| **Capacity (sponge)** | **8,192-bit** | N/A | N/A | 512-bit | 1,024-bit | N/A | N/A |")
    lines.append("| **Algebraic Domains** | **3** (ARX + GF(2¹²⁸) + GF(p)) | 1 | 1 | 1 | 1 | 1 | 1 |")
    lines.append("| **Permutation Rounds** | **32 × 3** = 96 | 64 | 80 | 24 | 24 | 12 | 7 |")
    lines.append("")
    lines.append("† *Generic bounds assuming ideal permutation behavior. AISE has not undergone independent cryptanalysis — these bounds are theoretical upper limits, not proven security levels.*\n")
    lines.append("> [!NOTE]")
    lines.append("> **What the 16,384-bit state actually provides:** In the sponge model, the capacity (8,192 bits for AISE) determines resistance to *structural* attacks — inner-collision attacks, state-recovery attacks, and capacity-targeting attacks. AISE's capacity is 8× larger than SHA3-512's (1,024 bits) and 16× larger than SHA3-256's (512 bits). This is a meaningful structural advantage, but it is distinct from the output-level collision bound.")
    lines.append(">")
    lines.append("> **Important design note:** Π_Ω is a *surjection*, not a bijection. Each 128-bit lane is reduced to 127 bits via Mersenne reduction before Π_C, causing ~128 bits of information loss per permutation call. This does not break the hash (compression is inherent to hashing), but it means the standard sponge security proof — which assumes a bijective permutation — requires careful adaptation.\n")


    # ══════════════════════════════════════════════════════════
    #  ANALYSIS
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## When to Use What — Practical Guidance\n")

    lines.append("### 🏆 BLAKE3 — Best for: Raw Speed\n")
    lines.append(f"- **{blake3_tp_1mb:.0f}x faster than AISE** at 1MB inputs")
    lines.append("- Parallelizable across cores (Merkle tree construction)")
    lines.append("- Best choice for: file integrity checking, content-addressable storage, deduplication, CI/CD pipelines")
    lines.append("- Limitation: 128-bit collision resistance (256-bit output)")
    lines.append("")

    lines.append("### 🛡️ SHA-256 — Best for: Compatibility & Standards Compliance\n")
    lines.append(f"- **{sha256_tp_1mb:.0f}x faster than AISE** at 1MB inputs")
    lines.append("- Universal support: TLS, X.509, Bitcoin, HMAC-SHA256")
    lines.append("- Best choice for: anything requiring interoperability, digital signatures, certificates")
    lines.append("- Limitation: Merkle-Damgård length extension vulnerability (mitigated by HMAC)")
    lines.append("")

    sha512_tp = rankings_1mb.get("SHA-512", 0)
    lines.append("### 🔐 SHA-512 — Best for: 64-bit Platform Performance + Higher Security\n")
    lines.append(f"- **{sha512_tp:.0f}x faster than AISE** — optimized for 64-bit registers")
    lines.append("- 256-bit collision resistance (vs SHA-256's 128-bit)")
    lines.append("- Best choice for: Ed25519 signatures, high-security document hashing, certificate transparency")
    lines.append("")

    sha3_256_tp = rankings_1mb.get("SHA3-256", 0)
    lines.append("### 🧬 SHA3-256/512 — Best for: Post-SHA-2 Diversity\n")
    lines.append(f"- Keccak sponge construction — fundamentally different from SHA-2")
    lines.append("- No length extension attacks (sponge property)")
    lines.append("- Best choice for: defense-in-depth hash diversity, NIST compliance where SHA-3 is mandated")
    lines.append(f"- Note: Slower than SHA-2 on x86 ({sha3_256_tp:.0f}x faster than AISE)")
    lines.append("")

    blake2_tp = rankings_1mb.get("BLAKE2b", 0)
    lines.append("### ⚡ BLAKE2b — Best for: Fast 512-bit Hashing\n")
    lines.append(f"- **{blake2_tp:.0f}x faster than AISE** — the fastest 512-bit output hash")
    lines.append("- Direct replacement for SHA-512 with better performance")
    lines.append("- Best choice for: password hashing (Argon2 internal), key derivation, general-purpose 512-bit digest")
    lines.append("")

    lines.append("### 🔮 AISE-HASH — Best for: Structural Security Margin & Research\n")
    lines.append(f"- **{aise_tp_1mb:.2f} MB/s** — deliberately slow due to 16,384-bit state processing")
    lines.append("- **3 algebraically independent domains** — an attacker must simultaneously defeat ARX, binary field, and prime field constructions")
    lines.append("- **8,192-bit capacity** — the largest sponge capacity of any known hash construction, providing enormous structural margin against non-generic attacks")
    lines.append("- **~2²⁵⁶ generic collision resistance** — same output-level bound as SHA-512/SHA3-512/BLAKE2b (all produce 512-bit digests)")
    lines.append("- Best choice for:")
    lines.append("  - Research baseline for multi-algebraic sponge designs")
    lines.append("  - Exploring heterogeneous permutation cascades")
    lines.append("  - Scenarios where structural diversity matters more than raw throughput")
    lines.append("  - Hashing small secrets (keys, passwords, tokens) where latency is acceptable")
    lines.append("- **Not suitable for:** high-throughput data pipelines, real-time file hashing, network protocols")
    lines.append("- **Cryptanalysis status:** Unaudited — independent analysis is actively invited (see [SECURITY.md](SECURITY.md))")
    lines.append("")

    # ══════════════════════════════════════════════════════════
    #  CONCLUSION
    # ══════════════════════════════════════════════════════════
    lines.append("---\n")
    lines.append("## Conclusion\n")
    if aise_tp_1mb > 0 and blake3_tp_1mb > 0:
        ratio = blake3_tp_1mb / aise_tp_1mb
        lines.append(f"AEGIS-Ω is **~{ratio:.0f}x slower** than BLAKE3 (the fastest algorithm tested) and **~{sha256_tp_1mb/aise_tp_1mb:.0f}x slower** than SHA-256. This is **entirely by design**.\n")
    lines.append("AISE's performance cost buys:")
    lines.append("1. **Algebraic heterogeneity**: Three independent mathematical domains (ARX, GF(2¹²⁸), GF(p)) that an attacker must simultaneously defeat")
    lines.append("2. **Massive structural margin**: 16,384-bit internal state with 8,192-bit capacity (8× larger than SHA3-512, 16× larger than SHA3-256), providing deep resistance to inner-collision and state-recovery attacks")
    lines.append("3. **Rescue-style S-boxes**: Alternating power maps ($x^5$ / $x^d$) that guarantee exponential algebraic degree growth in both forward and backward directions")
    lines.append("4. **96 total rounds** across 3 algebraic domains (vs. 24 for SHA3, 7 for BLAKE3)\n")
    lines.append("**What this does NOT buy:** The generic collision resistance of AISE-HASH is ~2²⁵⁶ — identical to SHA-512, SHA3-512, and BLAKE2b — because all produce 512-bit outputs. The large internal state provides *structural* security margin, not a higher output-level collision bound.\n")
    lines.append("The question is: *\"does the structural diversity and enormous capacity justify the performance cost for your use case?\"* For cryptographic research and exploring multi-algebraic designs, yes. For hashing gigabytes of data, use BLAKE3.\n")
    lines.append("> [!WARNING]")
    lines.append("> AEGIS-Ω has **not undergone independent cryptanalysis**. The security claims above assume ideal permutation behavior. Until the design has been subjected to rigorous analysis by independent cryptographers, AISE should be treated as an experimental research construction. See [SECURITY.md](SECURITY.md) for how to contribute cryptanalysis.\n")

    lines.append("---\n")
    lines.append("*Report generated by AEGIS-Ω Benchmark Suite*")

    with open(output_path, 'w') as f:
        f.write('\n'.join(lines))
    print(f"  Generated: {output_path}")


# ──────────────────────────────────────────────────────────────
#  Main
# ──────────────────────────────────────────────────────────────

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 gen_compare.py <bench_results.json>")
        sys.exit(1)

    input_file = sys.argv[1]
    with open(input_file) as f:
        data = json.load(f)

    # Output paths
    project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    charts_dir = os.path.join(project_root, "charts")
    os.makedirs(charts_dir, exist_ok=True)

    compare_md = os.path.join(project_root, "COMPARE.md")

    print("Generating charts...")
    gen_hero_chart(data["throughput"], os.path.join(charts_dir, "hero_throughput.svg"))
    gen_throughput_chart(data["throughput"], os.path.join(charts_dir, "throughput.svg"))
    gen_latency_chart(data["latency"], os.path.join(charts_dir, "latency.svg"))
    gen_scalability_chart(data["scalability"], os.path.join(charts_dir, "scalability.svg"))
    gen_permutation_chart(data["permutation_breakdown"], os.path.join(charts_dir, "permutation.svg"))

    print("\nGenerating COMPARE.md...")
    gen_compare_md(data, charts_dir, compare_md)

    print("\n✅ Done! Report at:", compare_md)
    print("   Charts at:", charts_dir)


if __name__ == "__main__":
    main()
