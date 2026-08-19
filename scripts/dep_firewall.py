#!/usr/bin/env python3
"""QUEUE #45 / Analyze-80: workspace Cargo graph firewall.

Rules (workspace packages only):
- no directed cycles on normal+build edges
- aira-core must not reach aira-node, aira-peer, or concrete CSU (any dep kind)
- concrete CSU crates must not depend on each other (any dep kind)

Exit 0 if the live graph is clean. --self-test proves violations fail closed.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from collections import defaultdict
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Set, Tuple

CORE = "aira-core"
FORBIDDEN_CORE_NAMES = frozenset({"aira-node", "aira-peer"})


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def is_concrete_csu(manifest_path: str, root: Path) -> bool:
    try:
        rel = Path(manifest_path).resolve().relative_to(root.resolve())
    except ValueError:
        return False
    return len(rel.parts) >= 2 and rel.parts[0] == "csu"


def load_metadata(root: Path) -> dict:
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--offline"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        proc = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--locked"],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr)
        raise SystemExit(proc.returncode or 1)
    return json.loads(proc.stdout)


def workspace_packages(metadata: dict) -> Dict[str, dict]:
    members = set(metadata["workspace_members"])
    out: Dict[str, dict] = {}
    for pkg in metadata["packages"]:
        if pkg["id"] in members:
            out[pkg["name"]] = pkg
    return out


def _dep_kinds(edge: dict) -> List[Optional[str]]:
    kinds = edge.get("dep_kinds")
    if kinds:
        return [k.get("kind") for k in kinds]
    return [None]


def edges_from_resolve(
    metadata: dict, names: Set[str]
) -> Tuple[Dict[str, Set[str]], Dict[str, Set[str]]]:
    """Return (prod_edges, all_kind_edges) as name → successor names."""
    id_to_name = {}
    for pkg in metadata["packages"]:
        id_to_name[pkg["id"]] = pkg["name"]

    prod: Dict[str, Set[str]] = defaultdict(set)
    all_kinds: Dict[str, Set[str]] = defaultdict(set)
    resolve = metadata.get("resolve") or {}
    for node in resolve.get("nodes") or []:
        src = id_to_name.get(node["id"])
        if src not in names:
            continue
        for dep in node.get("deps") or []:
            dst = dep.get("name")
            if dst not in names:
                continue
            kinds = _dep_kinds(dep)
            all_kinds[src].add(dst)
            if any(k in (None, "build") for k in kinds):
                prod[src].add(dst)
        if not node.get("deps") and node.get("dependencies"):
            for dep_id in node["dependencies"]:
                dst = id_to_name.get(dep_id)
                if dst in names:
                    prod[src].add(dst)
                    all_kinds[src].add(dst)
    for n in names:
        prod.setdefault(n, set())
        all_kinds.setdefault(n, set())
    return prod, all_kinds


def find_cycle(graph: Dict[str, Set[str]]) -> Optional[List[str]]:
    WHITE, GRAY, BLACK = 0, 1, 2
    color = {n: WHITE for n in graph}
    parent: Dict[str, Optional[str]] = {n: None for n in graph}

    def dfs(u: str) -> Optional[List[str]]:
        color[u] = GRAY
        for v in sorted(graph[u]):
            if color[v] == WHITE:
                parent[v] = u
                found = dfs(v)
                if found:
                    return found
            elif color[v] == GRAY:
                cycle = [v]
                cur = u
                while cur != v:
                    cycle.append(cur)
                    nxt = parent[cur]
                    if nxt is None:
                        break
                    cur = nxt
                cycle.append(v)
                cycle.reverse()
                return cycle
        color[u] = BLACK
        return None

    for n in sorted(graph):
        if color[n] == WHITE:
            found = dfs(n)
            if found:
                return found
    return None


def reachable(graph: Dict[str, Set[str]], start: str) -> Set[str]:
    seen: Set[str] = set()
    stack = [start]
    while stack:
        u = stack.pop()
        if u in seen:
            continue
        seen.add(u)
        stack.extend(graph.get(u, ()))
    seen.discard(start)
    return seen


def check_graph(
    prod: Dict[str, Set[str]],
    all_kinds: Dict[str, Set[str]],
    concrete: Set[str],
) -> List[str]:
    errors: List[str] = []
    cycle = find_cycle(prod)
    if cycle:
        errors.append("import cycle: " + " → ".join(cycle))

    if CORE in all_kinds:
        reach = reachable(all_kinds, CORE)
        forbidden = (FORBIDDEN_CORE_NAMES | concrete) & (reach | all_kinds.get(CORE, set()))
        for name in sorted(forbidden):
            errors.append(f"{CORE} ↛ forbidden package {name}")

    for src in sorted(concrete):
        for dst in sorted(all_kinds.get(src, ())):
            if dst in concrete:
                errors.append(f"CSU ↛ CSU: {src} → {dst}")
    return errors


def rust_crate_ident(pkg_name: str) -> str:
    return pkg_name.replace("-", "_")


def _uses_crate(text: str, ident: str) -> bool:
    return f"{ident}::" in text or f"use {ident}" in text or f"extern crate {ident}" in text


def scan_sources(root: Path, pkgs: Dict[str, dict], concrete: Iterable[str]) -> List[str]:
    errors: List[str] = []
    concrete_set = set(concrete)
    core_pkg = pkgs.get(CORE)
    if core_pkg:
        core_src = Path(core_pkg["manifest_path"]).parent / "src"
        forbidden_idents = [rust_crate_ident(n) for n in sorted(FORBIDDEN_CORE_NAMES | concrete_set)]
        if core_src.is_dir():
            for path in core_src.rglob("*.rs"):
                text = path.read_text(encoding="utf-8")
                for ident in forbidden_idents:
                    if _uses_crate(text, ident):
                        errors.append(
                            f"source import {path.relative_to(root)} uses forbidden {ident}"
                        )
    for name in sorted(concrete_set):
        pkg = pkgs.get(name)
        if not pkg:
            continue
        src = Path(pkg["manifest_path"]).parent / "src"
        own = rust_crate_ident(name)
        if not src.is_dir():
            continue
        text_files = list(src.rglob("*.rs"))
        for path in text_files:
            text = path.read_text(encoding="utf-8")
            for other in sorted(concrete_set):
                ident = rust_crate_ident(other)
                if ident == own:
                    continue
                if _uses_crate(text, ident):
                    errors.append(
                        f"source import {path.relative_to(root)} uses peer CSU {ident}"
                    )
    return errors


def evaluate_live(root: Path) -> List[str]:
    metadata = load_metadata(root)
    pkgs = workspace_packages(metadata)
    names = set(pkgs)
    concrete = {
        name
        for name, pkg in pkgs.items()
        if is_concrete_csu(pkg["manifest_path"], root)
    }
    prod, all_kinds = edges_from_resolve(metadata, names)
    errors = check_graph(prod, all_kinds, concrete)
    errors.extend(scan_sources(root, pkgs, concrete))
    return errors


def _assert(cond: bool, msg: str, failures: List[str]) -> None:
    if not cond:
        failures.append(msg)


def self_test() -> int:
    failures: List[str] = []
    concrete = {"aira-csu-execution-basic", "aira-csu-verification-basic"}

    prod = {
        CORE: {"aira-object"},
        "aira-object": set(),
        "aira-node": {CORE},
        "aira-csu-execution-basic": set(),
        "aira-csu-verification-basic": set(),
    }
    all_kinds = {k: set(v) for k, v in prod.items()}
    _assert(
        check_graph(prod, all_kinds, concrete) == [],
        "clean graph must pass",
        failures,
    )

    bad_core = {k: set(v) for k, v in all_kinds.items()}
    bad_core[CORE].add("aira-node")
    errs = check_graph(prod, bad_core, concrete)
    _assert(
        any("aira-node" in e for e in errs),
        f"core→node must fail, got {errs}",
        failures,
    )

    trans = {k: set(v) for k, v in all_kinds.items()}
    trans[CORE].add("aira-flow")
    trans["aira-flow"] = {"aira-csu-execution-basic"}
    trans.setdefault("aira-csu-execution-basic", set())
    errs = check_graph(prod, trans, concrete)
    _assert(
        any("aira-csu-execution-basic" in e for e in errs),
        f"transitive core→CSU must fail, got {errs}",
        failures,
    )

    csu_edge = {k: set(v) for k, v in all_kinds.items()}
    csu_edge["aira-csu-execution-basic"].add("aira-csu-verification-basic")
    prod_csu = {k: set(v) for k, v in prod.items()}
    prod_csu["aira-csu-execution-basic"].add("aira-csu-verification-basic")
    errs = check_graph(prod_csu, csu_edge, concrete)
    _assert(
        any("CSU ↛ CSU" in e for e in errs),
        f"CSU→CSU must fail, got {errs}",
        failures,
    )

    cyclic = {
        "a": {"b"},
        "b": {"a"},
        CORE: set(),
    }
    errs = check_graph(cyclic, cyclic, set())
    _assert(
        any("import cycle" in e for e in errs),
        f"cycle must fail, got {errs}",
        failures,
    )

    if failures:
        for f in failures:
            print(f"SELFTEST FAIL: {f}", file=sys.stderr)
        return 1
    print("dep_firewall self-test: ok")
    return 0


def main(argv: List[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    root = repo_root()
    os.chdir(root)
    errors = evaluate_live(root)
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print("dep_firewall: workspace graph is clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
