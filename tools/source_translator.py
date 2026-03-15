#!/usr/bin/env python3
"""
TrustOS Source Code Translator
==============================

Transforms Rust source code with different presets while preserving exact structure.

Presets:
  original        - No changes (identity copy)
  minimal         - Minimize text bytes: shortest names, strip comments
  educational     - Expanded names, detailed annotations (--lang en|fr)

Usage:
  # Minimal preset — strip comments, shorten all identifiers
  python source_translator.py --preset minimal -i kernel/src/ -o translated/minimal/

  # Educational English — expand abbreviations, add learning annotations
  python source_translator.py --preset educational --lang en -i kernel/src/ -o translated/edu-en/

  # Educational French — same but annotations in French
  python source_translator.py --preset educational --lang fr -i kernel/src/ -o translated/edu-fr/

  # Generate mapping file only (dry-run)
  python source_translator.py --preset minimal -i kernel/src/ --dry-run --save-mapping mapping.json

  # Apply a previously-edited mapping
  python source_translator.py --preset minimal -i kernel/src/ -o translated/ --load-mapping mapping.json
"""

import re
import os
import sys
import json
import argparse
import shutil
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple, Set
from collections import Counter

# ════════════════════════════════════════════════════════════════
#  CONSTANTS
# ════════════════════════════════════════════════════════════════

RUST_KEYWORDS = frozenset({
    'as', 'async', 'await', 'break', 'const', 'continue', 'crate', 'dyn',
    'else', 'enum', 'extern', 'false', 'fn', 'for', 'if', 'impl', 'in',
    'let', 'loop', 'match', 'mod', 'move', 'mut', 'pub', 'ref', 'return',
    'self', 'Self', 'static', 'struct', 'super', 'trait', 'true', 'type',
    'unsafe', 'use', 'where', 'while', 'yield', 'abstract', 'become',
    'box', 'do', 'final', 'macro', 'override', 'priv', 'try', 'typeof',
    'unsized', 'virtual',
})

RUST_STD_TYPES = frozenset({
    'bool', 'char', 'str', 'u8', 'u16', 'u32', 'u64', 'u128', 'usize',
    'i8', 'i16', 'i32', 'i64', 'i128', 'isize', 'f32', 'f64',
    'Option', 'Some', 'None', 'Result', 'Ok', 'Err',
    'Vec', 'String', 'Box', 'Rc', 'Arc', 'Cell', 'RefCell',
    'HashMap', 'HashSet', 'BTreeMap', 'BTreeSet', 'VecDeque',
    'Iterator', 'IntoIterator', 'FromIterator',
    'Display', 'Debug', 'Clone', 'Copy', 'Default', 'Drop',
    'Send', 'Sync', 'Sized', 'Unpin',
    'From', 'Into', 'TryFrom', 'TryInto',
    'AsRef', 'AsMut', 'Borrow', 'BorrowMut',
    'Fn', 'FnMut', 'FnOnce',
    'Read', 'Write', 'Seek', 'BufRead',
    'Add', 'Sub', 'Mul', 'Div', 'Rem', 'Neg',
    'Deref', 'DerefMut', 'Index', 'IndexMut',
    'PartialEq', 'Eq', 'PartialOrd', 'Ord', 'Hash',
    'Formatter', 'Arguments',
    'PhantomData', 'ManuallyDrop', 'MaybeUninit',
    'Pin', 'Future', 'Poll', 'Waker',
    'AtomicBool', 'AtomicU8', 'AtomicU16', 'AtomicU32', 'AtomicU64',
    'AtomicUsize', 'AtomicI8', 'AtomicI16', 'AtomicI32', 'AtomicI64',
    'AtomicIsize', 'Ordering', 'Mutex', 'RwLock', 'Once',
    'NonZeroU8', 'NonZeroU16', 'NonZeroU32', 'NonZeroU64', 'NonZeroUsize',
})

RUST_STD_MACROS = frozenset({
    'println', 'print', 'eprintln', 'eprint', 'format', 'write', 'writeln',
    'vec', 'assert', 'assert_eq', 'assert_ne', 'debug_assert',
    'debug_assert_eq', 'debug_assert_ne', 'cfg', 'env', 'concat',
    'stringify', 'include', 'include_str', 'include_bytes',
    'file', 'line', 'column', 'module_path', 'todo', 'unimplemented',
    'unreachable', 'panic', 'compile_error', 'format_args',
    'log', 'log_debug', 'log_warn', 'log_error',
    'serial_print', 'serial_println',
})

# Identifiers that must never be renamed
PROTECTED_IDENTS = RUST_KEYWORDS | RUST_STD_TYPES | RUST_STD_MACROS | frozenset({
    'self', 'Self', 'super', 'crate',
    'main', '_start', '_',
    'core', 'alloc', 'std',
    'spin', 'volatile', 'bitflags', 'lazy_static', 'x86_64',
    'test', 'cfg', 'allow', 'deny', 'warn', 'derive',
    'feature', 'target_arch', 'not',
    'link_section', 'no_mangle', 'repr', 'packed', 'align',
    'C', 'transparent', 'Rust',
    'inline', 'always', 'never', 'cold',
    'must_use', 'deprecated', 'doc', 'hidden',
    'global_allocator', 'panic_handler', 'alloc_error_handler',
    'no_std', 'no_main',
    'asm', 'global_asm',
    'size_of', 'align_of',
    'transmute', 'zeroed',
    'drop', 'forget',
    'null', 'null_mut',
    'Relaxed', 'Release', 'Acquire', 'AcqRel', 'SeqCst',
    'Serialize', 'Deserialize',
    # macro-related
    'macro_rules', 'macro_export',
    # macro fragment specifiers
    'tt', 'expr', 'ident', 'path', 'ty', 'pat', 'pat_param',
    'stmt', 'block', 'item', 'meta', 'vis', 'literal', 'lifetime',
    # common std module paths
    'fmt', 'io', 'ops', 'mem', 'ptr', 'slice', 'iter', 'convert',
    'collections', 'sync', 'cell', 'marker', 'any', 'num', 'cmp', 'hash',
    'option', 'result', 'string', 'borrow', 'rc', 'pin', 'task',
    'atomic', 'hint', 'intrinsics',
    # common crate names
    'serde', 'log', 'limine',
    # common method names from std traits (must match trait definition)
    'new', 'default', 'clone', 'fmt', 'drop', 'deref',
    'into', 'from', 'try_into', 'try_from',
    'as_ref', 'as_mut', 'borrow', 'borrow_mut',
    'write_str', 'write_fmt', 'write_char',
    'next', 'next_back', 'size_hint',
    'poll', 'wake',
    'eq', 'ne', 'lt', 'le', 'gt', 'ge', 'cmp', 'partial_cmp',
    'hash', 'index', 'index_mut',
    'add', 'sub', 'mul', 'div', 'rem', 'neg', 'not',
    'bitand', 'bitor', 'bitxor', 'shl', 'shr',
    'expect', 'unwrap', 'unwrap_or', 'unwrap_or_else',
    'map', 'and_then', 'or_else', 'ok_or', 'ok_or_else',
    'is_some', 'is_none', 'is_ok', 'is_err',
    'len', 'is_empty', 'contains', 'get',
    'push', 'pop', 'insert', 'remove', 'clear',
    'lock', 'try_lock', 'read', 'write',
    'load', 'store', 'fetch_add', 'fetch_sub', 'compare_exchange',
    'as_bytes', 'as_str', 'to_string', 'to_owned',
    # stdlib alloc functions (must not be renamed)
    'alloc_zeroed', 'alloc_layout', 'dealloc', 'realloc',
    'handle_alloc_error',
    # Rust attributes and built-in macros
    'cfg_attr', 'cfg_if', 'cfg_accessible', 'derive_const',
    'bench', 'global_allocator', 'test_case',
    'include_bytes', 'include_str', 'concat_idents',
    # External trait methods (from dependencies)
    'draw_iter', 'draw_pixel', 'fill_solid', 'fill_contiguous',
    'bounding_box', 'color_converted',
    # asm-related identifiers
    'reg_byte', 'inout', 'lateout', 'inlateout', 'nomem', 'nostack',
    'preserves_flags', 'att_syntax', 'options',
    'rax', 'rbx', 'rcx', 'rdx', 'rsi', 'rdi', 'rsp', 'rbp',
    'r8', 'r9', 'r10', 'r11', 'r12', 'r13', 'r14', 'r15',
    'eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'esp', 'ebp',
    'ax', 'bx', 'cx', 'dx', 'si', 'di', 'sp', 'bp', 'al', 'bl', 'cl', 'dl',
    # Common unsafe/FFI function names
    'write_volatile', 'read_volatile', 'copy_nonoverlapping',
    'swap', 'replace', 'take',
})

# ── Abbreviation expansion dictionary (for educational mode) ──

ABBREV_EXPAND = {
    'buf': 'buffer', 'ctx': 'context', 'msg': 'message', 'addr': 'address',
    'len': 'length', 'ptr': 'pointer', 'idx': 'index', 'cnt': 'count',
    'val': 'value', 'ret': 'return_value', 'src': 'source', 'dst': 'destination',
    'prev': 'previous', 'curr': 'current', 'desc': 'descriptor',
    'alloc': 'allocator',
    'init': 'initialize', 'deinit': 'deinitialize',
    'fmt': 'formatter', 'disp': 'display', 'cb': 'callback',
    'err': 'error', 'res': 'result', 'req': 'request', 'resp': 'response',
    'cmd': 'command', 'pkt': 'packet', 'hdr': 'header', 'tbl': 'table',
    'freq': 'frequency', 'info': 'information', 'stat': 'status',
    'mem': 'memory', 'reg': 'register',
    'irq': 'interrupt_request', 'isr': 'interrupt_handler',
    'hw': 'hardware', 'sw': 'software',
    'tx': 'transmit', 'rx': 'receive', 'ack': 'acknowledge',
    'fb': 'framebuffer', 'kb': 'keyboard', 'ms': 'mouse',
    'sched': 'scheduler', 'tsk': 'task', 'thd': 'thread', 'proc': 'process',
    'drv': 'driver', 'dev': 'device', 'fs': 'filesystem', 'dir': 'directory',
    'sec': 'sector', 'blk': 'block', 'pg': 'page', 'frm': 'frame',
    'stk': 'stack', 'prio': 'priority',
    'phy': 'physical', 'phys': 'physical',
    'sz': 'size', 'nr': 'number', 'num': 'number',
    'tmp': 'temporary', 'temp': 'temporary',
    'pos': 'position', 'neg': 'negative', 'abs': 'absolute', 'rel': 'relative',
    'op': 'operation', 'ops': 'operations',
    'exc': 'exception', 'sig': 'signal', 'sem': 'semaphore',
    'cond': 'condition', 'ev': 'event', 'evt': 'event',
    'attr': 'attribute', 'elem': 'element', 'iter': 'iterator',
    'gen': 'generator', 'instr': 'instruction', 'exec': 'execute',
    'sys': 'system', 'usr': 'user', 'perm': 'permission',
    'cap': 'capability', 'seq': 'sequence',
    'rng': 'random_generator', 'rand': 'random',
    'enc': 'encrypt', 'dec': 'decrypt', 'auth': 'authenticate',
    'sess': 'session', 'tok': 'token',
    'delim': 'delimiter', 'sep': 'separator',
    'pfx': 'prefix', 'sfx': 'suffix',
    'hndl': 'handle', 'hdl': 'handle',
    'mgr': 'manager', 'ctl': 'controller', 'ctrl': 'controller',
    'srv': 'server', 'cli': 'client', 'conn': 'connection',
    'sock': 'socket', 'proto': 'protocol',
    'conf': 'configuration', 'param': 'parameter', 'arg': 'argument',
    'acc': 'accumulator', 'ctr': 'counter', 'dbg': 'debug', 'lvl': 'level',
    'cfg': 'configuration', 'tgt': 'target', 'obj': 'object',
    'wnd': 'window', 'btn': 'button', 'lbl': 'label', 'txt': 'text',
    'img': 'image', 'bmp': 'bitmap', 'px': 'pixel',
    'col': 'column', 'ln': 'line', 'ch': 'character',
    'max': 'maximum', 'min': 'minimum', 'avg': 'average',
}

# ── Educational annotations (pattern -> comment) ──

ANNOTATIONS_EN = {
    'unsafe_block':   '// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.',
    'static_mutex':   '// Global shared state guarded by a Mutex (mutual exclusion lock).',
    'static_atomic':  '// Atomic variable — provides lock-free thread-safe access.',
    'pub_fn':         '// Public function — callable from other modules.',
    'pub_struct':     '// Public structure — visible outside this module.',
    'impl_block':     '// Implementation block — defines methods for the type above.',
    'trait_impl':     '// Trait implementation — fulfills a behavioral contract.',
    'trait_def':      '// Trait definition — declares a shared interface that types can implement.',
    'enum_def':       '// Enumeration — a type that can be one of several variants.',
    'match_expr':     '// Pattern matching — Rust\'s exhaustive branching construct.',
    'loop_inf':       '// Infinite loop — runs until an explicit `break`.',
    'closure':        '// Closure — an anonymous function that captures its environment.',
    'derive_attr':    '// #[derive] — auto-generates trait implementations at compile time.',
    'no_std':         '// #![no_std] — this crate does not link the standard library (bare-metal).',
    'no_main':        '// #![no_main] — custom entry point instead of fn main().',
    'const_val':      '// Compile-time constant — evaluated at compilation, zero runtime cost.',
    'type_alias':     '// Type alias — gives an existing type a new name for clarity.',
    'lifetime':       '// Lifetime annotation — tells the compiler how long a reference is valid.',
    'generic':        '// Generic parameter — allows this code to work with multiple types.',
}

ANNOTATIONS_FR = {
    'unsafe_block':   '// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.',
    'static_mutex':   '// État global partagé protégé par un Mutex (verrou d\'exclusion mutuelle).',
    'static_atomic':  '// Variable atomique — accès thread-safe sans verrou.',
    'pub_fn':         '// Fonction publique — appelable depuis d\'autres modules.',
    'pub_struct':     '// Structure publique — visible à l\'extérieur de ce module.',
    'impl_block':     '// Bloc d\'implémentation — définit les méthodes du type ci-dessus.',
    'trait_impl':     '// Implémentation de trait — remplit un contrat comportemental.',
    'trait_def':      '// Définition de trait — déclare une interface partagée que les types peuvent implémenter.',
    'enum_def':       '// Énumération — un type qui peut être l\'une de plusieurs variantes.',
    'match_expr':     '// Correspondance de motifs — branchement exhaustif de Rust.',
    'loop_inf':       '// Boucle infinie — tourne jusqu\'à un `break` explicite.',
    'closure':        '// Fermeture (closure) — fonction anonyme qui capture son environnement.',
    'derive_attr':    '// #[derive] — génère automatiquement les implémentations de traits à la compilation.',
    'no_std':         '// #![no_std] — ce crate ne lie pas la bibliothèque standard (bare-metal).',
    'no_main':        '// #![no_main] — point d\'entrée personnalisé au lieu de fn main().',
    'const_val':      '// Constante de compilation — évaluée à la compilation, coût zéro à l\'exécution.',
    'type_alias':     '// Alias de type — donne un nouveau nom à un type existant pour la clarté.',
    'lifetime':       '// Annotation de durée de vie (lifetime) — indique au compilateur la durée de validité d\'une référence.',
    'generic':        '// Paramètre générique — permet à ce code de fonctionner avec plusieurs types.',
}


# ════════════════════════════════════════════════════════════════
#  TOKENS
# ════════════════════════════════════════════════════════════════

class TT:
    """Token types."""
    BLOCK_COMMENT  = 'block_comment'
    DOC_COMMENT    = 'doc_comment'
    LINE_COMMENT   = 'line_comment'
    RAW_STRING     = 'raw_string'
    BYTE_STRING    = 'byte_string'
    STRING         = 'string'
    CHAR           = 'char'
    LIFETIME       = 'lifetime'
    NUMBER         = 'number'
    KEYWORD        = 'keyword'
    IDENT          = 'ident'
    OPERATOR       = 'operator'
    WHITESPACE     = 'whitespace'
    UNKNOWN        = 'unknown'


@dataclass
class Token:
    type: str
    value: str
    line: int
    col: int


# ════════════════════════════════════════════════════════════════
#  TOKENIZER
# ════════════════════════════════════════════════════════════════

class RustTokenizer:
    """
    Converts Rust source text into a list of tokens.
    Concatenating all token values reproduces the original source exactly.
    """

    def __init__(self, source: str):
        self.src = source
        self.pos = 0
        self.line = 1
        self.col = 1

    # ── public ──────────────────────────────────────────────

    def tokenize(self) -> List[Token]:
        tokens: List[Token] = []
        while self.pos < len(self.src):
            t = self._next()
            if t:
                tokens.append(t)
        return tokens

    # ── helpers ─────────────────────────────────────────────

    def _ch(self, offset=0) -> str:
        p = self.pos + offset
        return self.src[p] if p < len(self.src) else '\0'

    def _advance(self, n=1):
        for _ in range(n):
            if self.pos < len(self.src):
                if self.src[self.pos] == '\n':
                    self.line += 1
                    self.col = 1
                else:
                    self.col += 1
                self.pos += 1

    def _slice(self, start: int) -> str:
        return self.src[start:self.pos]

    # ── dispatch ────────────────────────────────────────────

    def _next(self) -> Optional[Token]:
        c = self._ch()
        sl, sc = self.line, self.col

        # whitespace
        if c in ' \t\r\n':
            return self._ws(sl, sc)

        # comments
        if c == '/' and self._ch(1) == '/':
            return self._line_comment(sl, sc)
        if c == '/' and self._ch(1) == '*':
            return self._block_comment(sl, sc)

        # raw / byte-raw strings
        if c == 'r' and self._ch(1) in ('"', '#'):
            t = self._raw_string(sl, sc)
            if t:
                return t
        if c == 'b':
            if self._ch(1) == 'r' and self._ch(2) in ('"', '#'):
                t = self._raw_string(sl, sc, byte=True)
                if t:
                    return t
            if self._ch(1) == '"':
                return self._string(sl, sc, prefix=1)
            if self._ch(1) == '\'':
                return self._byte_char(sl, sc)

        # strings
        if c == '"':
            return self._string(sl, sc)

        # char / lifetime
        if c == '\'':
            return self._char_or_lt(sl, sc)

        # numbers
        if c.isdigit():
            return self._number(sl, sc)

        # identifiers / keywords
        if c.isalpha() or c == '_':
            return self._ident(sl, sc)

        # everything else (operators, punctuation)
        return self._operator(sl, sc)

    # ── whitespace ──────────────────────────────────────────

    def _ws(self, l, c) -> Token:
        s = self.pos
        while self.pos < len(self.src) and self.src[self.pos] in ' \t\r\n':
            self._advance()
        return Token(TT.WHITESPACE, self._slice(s), l, c)

    # ── comments ────────────────────────────────────────────

    def _line_comment(self, l, c) -> Token:
        s = self.pos
        self._advance(2)  # //
        is_doc = (self._ch() == '/' and self._ch(1) != '/') or self._ch() == '!'
        while self.pos < len(self.src) and self.src[self.pos] != '\n':
            self._advance()
        return Token(TT.DOC_COMMENT if is_doc else TT.LINE_COMMENT, self._slice(s), l, c)

    def _block_comment(self, l, c) -> Token:
        s = self.pos
        self._advance(2)  # /*
        is_doc = (self._ch() == '*' and self._ch(1) != '/') or self._ch() == '!'
        depth = 1
        while self.pos < len(self.src) and depth > 0:
            if self._ch() == '/' and self._ch(1) == '*':
                depth += 1
                self._advance(2)
            elif self._ch() == '*' and self._ch(1) == '/':
                depth -= 1
                self._advance(2)
            else:
                self._advance()
        return Token(TT.DOC_COMMENT if is_doc else TT.BLOCK_COMMENT, self._slice(s), l, c)

    # ── strings ─────────────────────────────────────────────

    def _string(self, l, c, prefix=0) -> Token:
        s = self.pos
        self._advance(prefix + 1)  # skip prefix + opening "
        while self.pos < len(self.src):
            ch = self.src[self.pos]
            if ch == '\\':
                self._advance(2)
            elif ch == '"':
                self._advance()
                break
            else:
                self._advance()
        tt = TT.BYTE_STRING if prefix else TT.STRING
        return Token(tt, self._slice(s), l, c)

    def _raw_string(self, l, c, byte=False) -> Optional[Token]:
        s = self.pos
        saved_pos, saved_line, saved_col = self.pos, self.line, self.col
        self._advance(2 if byte else 1)  # skip b?r
        hashes = 0
        while self.pos < len(self.src) and self.src[self.pos] == '#':
            hashes += 1
            self._advance()
        if self._ch() != '"':
            self.pos, self.line, self.col = saved_pos, saved_line, saved_col
            return None
        self._advance()  # opening "
        closing = '"' + '#' * hashes
        while self.pos < len(self.src):
            if self.src[self.pos:self.pos + len(closing)] == closing:
                self._advance(len(closing))
                tt = TT.BYTE_STRING if byte else TT.RAW_STRING
                return Token(tt, self._slice(s), l, c)
            self._advance()
        tt = TT.BYTE_STRING if byte else TT.RAW_STRING
        return Token(tt, self._slice(s), l, c)

    def _byte_char(self, l, c) -> Token:
        s = self.pos
        self._advance(2)  # b'
        if self._ch() == '\\':
            self._advance(2)
        else:
            self._advance()
        if self._ch() == '\'':
            self._advance()
        return Token(TT.CHAR, self._slice(s), l, c)

    def _char_or_lt(self, l, c) -> Token:
        s = self.pos
        self._advance()  # skip '
        if self._ch().isalpha() or self._ch() == '_':
            id_start = self.pos
            while self.pos < len(self.src) and (self.src[self.pos].isalnum() or self.src[self.pos] == '_'):
                self._advance()
            if self._ch() == '\'':
                # char literal like 'a' or 'X'
                if self.pos - id_start == 1:
                    self._advance()
                    return Token(TT.CHAR, self._slice(s), l, c)
            # lifetime 'ident
            return Token(TT.LIFETIME, self._slice(s), l, c)
        elif self._ch() == '\\':
            self._advance()
            if self._ch():
                self._advance()
            if self._ch() == '\'':
                self._advance()
            return Token(TT.CHAR, self._slice(s), l, c)
        elif self._ch() and self._ch() != '\'' and self._ch(1) == '\'':
            # Non-alphanumeric char literal: '"', ' ', '!', '.', '/', etc.
            self._advance()  # consume the char
            self._advance()  # consume closing '
            return Token(TT.CHAR, self._slice(s), l, c)
        return Token(TT.OPERATOR, self._slice(s), l, c)

    # ── numbers ─────────────────────────────────────────────

    def _number(self, l, c) -> Token:
        s = self.pos
        if self._ch() == '0' and self._ch(1) in 'xXoObB':
            self._advance(2)
            while self.pos < len(self.src) and (self.src[self.pos] in '0123456789abcdefABCDEF_'):
                self._advance()
        else:
            while self.pos < len(self.src) and (self.src[self.pos].isdigit() or self.src[self.pos] == '_'):
                self._advance()
            if self._ch() == '.' and self._ch(1).isdigit():
                self._advance()
                while self.pos < len(self.src) and (self.src[self.pos].isdigit() or self.src[self.pos] == '_'):
                    self._advance()
            if self._ch() in 'eE':
                self._advance()
                if self._ch() in '+-':
                    self._advance()
                while self.pos < len(self.src) and (self.src[self.pos].isdigit() or self.src[self.pos] == '_'):
                    self._advance()
        # type suffix
        for suf in ('u128','i128','usize','isize','u64','i64','u32','i32','u16','i16','u8','i8','f64','f32'):
            if self.src[self.pos:self.pos+len(suf)] == suf:
                self._advance(len(suf))
                break
        return Token(TT.NUMBER, self._slice(s), l, c)

    # ── identifiers / keywords ──────────────────────────────

    def _ident(self, l, c) -> Token:
        s = self.pos
        while self.pos < len(self.src) and (self.src[self.pos].isalnum() or self.src[self.pos] == '_'):
            self._advance()
        v = self._slice(s)
        if v in RUST_KEYWORDS:
            return Token(TT.KEYWORD, v, l, c)
        return Token(TT.IDENT, v, l, c)

    # ── operators / punctuation ─────────────────────────────

    MULTI_OPS = frozenset({
        '...', '<<=', '>>=', '..=',
        '->', '=>', '::', '..', '==', '!=', '<=', '>=',
        '&&', '||', '<<', '>>', '+=', '-=', '*=', '/=',
        '%=', '&=', '|=', '^=',
    })

    def _operator(self, l, c) -> Token:
        s = self.pos
        tri = self.src[self.pos:self.pos+3]
        duo = self.src[self.pos:self.pos+2]
        if tri in self.MULTI_OPS:
            self._advance(3)
        elif duo in self.MULTI_OPS:
            self._advance(2)
        else:
            self._advance()
        return Token(TT.OPERATOR, self._slice(s), l, c)


# ════════════════════════════════════════════════════════════════
#  NAME GENERATORS
# ════════════════════════════════════════════════════════════════

def _idx_to_name(idx: int, alpha: str) -> str:
    """Convert integer index to letter-based name: 0->a, 25->z, 26->aa, …"""
    if idx < 26:
        return alpha[idx]
    return _idx_to_name(idx // 26 - 1, alpha) + alpha[idx % 26]


class MinimalNameGen:
    """Generates the shortest possible unique names per naming convention.
    Tracks all generated names globally to avoid cross-style collisions."""

    def __init__(self):
        self._counters = {'snake': 0, 'pascal': 0, 'upper': 0}
        self._used: Set[str] = set()

    def next_name(self, style: str) -> str:
        while True:
            idx = self._counters[style]
            self._counters[style] += 1
            if style == 'snake':
                name = _idx_to_name(idx, 'abcdefghijklmnopqrstuvwxyz')
            elif style == 'pascal':
                raw = _idx_to_name(idx, 'abcdefghijklmnopqrstuvwxyz')
                name = raw[0].upper() + raw[1:]
            else:  # upper / SCREAMING_SNAKE
                raw = _idx_to_name(idx, 'abcdefghijklmnopqrstuvwxyz')
                name = raw.upper() + '_'
            if name not in PROTECTED_IDENTS and name not in self._used:
                self._used.add(name)
                return name


def _classify_ident(name: str) -> str:
    """Classify an identifier into snake / pascal / upper naming style."""
    if '_' in name and name == name.upper():
        return 'upper'
    if name[0].isupper():
        return 'pascal'
    return 'snake'


# ── Educational name expansion ──────────────────────────────

def _split_snake(name: str) -> List[str]:
    return name.split('_')


def _split_pascal(name: str) -> List[str]:
    parts = re.findall(r'[A-Z][a-z0-9]*|[a-z][a-z0-9]*|[A-Z]+(?=[A-Z][a-z]|\b)', name)
    return parts if parts else [name]


def _expand_parts(parts: List[str]) -> List[str]:
    expanded = []
    for p in parts:
        low = p.lower()
        expanded.append(ABBREV_EXPAND.get(low, low))
    return expanded


# Reserved and strict keywords in Rust that must never appear as identifiers
_RUST_RESERVED = frozenset({
    'abstract', 'become', 'box', 'do', 'final', 'macro', 'override',
    'priv', 'try', 'typeof', 'unsized', 'virtual', 'yield',
    'async', 'await', 'dyn',
})


def expand_identifier(name: str) -> str:
    """Expand abbreviated identifier to a more readable version.
    Returns the original name if expansion would produce a reserved keyword."""
    style = _classify_ident(name)
    if style == 'snake':
        parts = _split_snake(name)
        expanded = _expand_parts(parts)
        result = '_'.join(expanded)
    elif style == 'pascal':
        parts = _split_pascal(name)
        expanded = _expand_parts(parts)
        result = ''.join(w.capitalize() for w in expanded)
    else:  # UPPER
        parts = _split_snake(name)
        expanded = _expand_parts(parts)
        result = '_'.join(w.upper() for w in expanded)
    # Safety: reject if any part became a reserved keyword
    for part in expanded:
        if part.lower() in _RUST_RESERVED:
            return name
    return result


# ════════════════════════════════════════════════════════════════
#  CONTEXT ANALYSIS
# ════════════════════════════════════════════════════════════════

def collect_module_names(all_tokens: Dict[str, List[Token]]) -> Set[str]:
    """Scan all files for `mod xxx` declarations and path-prefix identifiers."""
    modules: Set[str] = set()
    for tokens in all_tokens.values():
        for i, tok in enumerate(tokens):
            # mod declarations
            if tok.type == TT.KEYWORD and tok.value == 'mod':
                for j in range(i + 1, min(i + 4, len(tokens))):
                    if tokens[j].type == TT.WHITESPACE:
                        continue
                    if tokens[j].type == TT.IDENT:
                        modules.add(tokens[j].value)
                    break
            # Path prefixes (identifiers followed by ::)
            if tok.type == TT.IDENT and is_path_prefix(tokens, i):
                modules.add(tok.value)
    return modules


def is_path_prefix(tokens: List[Token], idx: int) -> bool:
    """Check if token at idx is followed by :: (making it a module/crate path prefix)."""
    for j in range(idx + 1, min(idx + 3, len(tokens))):
        if tokens[j].type == TT.WHITESPACE:
            continue
        return tokens[j].type == TT.OPERATOR and tokens[j].value == '::'
    return False


def is_after_dot(tokens: List[Token], idx: int) -> bool:
    """Check if token at idx is preceded by . (method call)."""
    for j in range(idx - 1, max(idx - 3, -1), -1):
        if tokens[j].type == TT.WHITESPACE:
            continue
        return tokens[j].type == TT.OPERATOR and tokens[j].value == '.'
    return False


def detect_annotation_points(tokens: List[Token]) -> List[Tuple[int, str]]:
    """
    Scan token list and return (index, annotation_key) pairs
    for places where an educational annotation should be inserted.
    The annotation should be inserted BEFORE the token at `index`.
    """
    points: List[Tuple[int, str]] = []
    i = 0
    while i < len(tokens):
        tok = tokens[i]

        # unsafe { ... }
        if tok.type == TT.KEYWORD and tok.value == 'unsafe':
            if not _has_preceding_comment(tokens, i):
                points.append((i, 'unsafe_block'))

        # pub fn
        if tok.type == TT.KEYWORD and tok.value == 'pub':
            nk = _next_keyword(tokens, i)
            if nk == 'fn' and not _has_preceding_comment(tokens, i):
                points.append((i, 'pub_fn'))
            elif nk == 'struct' and not _has_preceding_comment(tokens, i):
                points.append((i, 'pub_struct'))
            elif nk == 'enum' and not _has_preceding_comment(tokens, i):
                points.append((i, 'enum_def'))
            elif nk == 'trait' and not _has_preceding_comment(tokens, i):
                points.append((i, 'trait_def'))

        # static ... Mutex / Atomic
        if tok.type == TT.KEYWORD and tok.value == 'static':
            text_ahead = _text_until_semi(tokens, i)
            if not _has_preceding_comment(tokens, i):
                if 'Mutex' in text_ahead or 'RwLock' in text_ahead:
                    points.append((i, 'static_mutex'))
                elif 'Atomic' in text_ahead:
                    points.append((i, 'static_atomic'))

        # impl ... for (trait impl) vs impl (inherent)
        if tok.type == TT.KEYWORD and tok.value == 'impl':
            if not _has_preceding_comment(tokens, i):
                text_ahead = _text_until_brace(tokens, i)
                if ' for ' in text_ahead:
                    points.append((i, 'trait_impl'))
                else:
                    points.append((i, 'impl_block'))

        # match
        if tok.type == TT.KEYWORD and tok.value == 'match':
            if not _has_preceding_comment(tokens, i):
                points.append((i, 'match_expr'))

        # loop { (infinite)
        if tok.type == TT.KEYWORD and tok.value == 'loop':
            if not _has_preceding_comment(tokens, i):
                points.append((i, 'loop_inf'))

        # const
        if tok.type == TT.KEYWORD and tok.value == 'const':
            nk = _next_keyword(tokens, i)
            if nk is None and not _has_preceding_comment(tokens, i):
                points.append((i, 'const_val'))

        # type alias
        if tok.type == TT.KEYWORD and tok.value == 'type':
            if not _has_preceding_comment(tokens, i):
                points.append((i, 'type_alias'))

        # #![no_std] / #![no_main]
        if tok.type == TT.OPERATOR and tok.value == '#':
            text = _text_until_bracket_close(tokens, i)
            if 'no_std' in text and not _has_preceding_comment(tokens, i):
                points.append((i, 'no_std'))
            elif 'no_main' in text and not _has_preceding_comment(tokens, i):
                points.append((i, 'no_main'))
            elif 'derive' in text and not _has_preceding_comment(tokens, i):
                points.append((i, 'derive_attr'))

        i += 1
    return points


def _has_preceding_comment(tokens: List[Token], idx: int) -> bool:
    """Check if there's a comment right before this token (skipping whitespace)."""
    for j in range(idx - 1, max(idx - 4, -1), -1):
        if tokens[j].type == TT.WHITESPACE:
            continue
        return tokens[j].type in (TT.LINE_COMMENT, TT.DOC_COMMENT, TT.BLOCK_COMMENT)
    return False


def _next_keyword(tokens: List[Token], idx: int) -> Optional[str]:
    for j in range(idx + 1, min(idx + 6, len(tokens))):
        if tokens[j].type == TT.WHITESPACE:
            continue
        if tokens[j].type == TT.KEYWORD:
            return tokens[j].value
        return None
    return None


def _text_until_semi(tokens: List[Token], idx: int) -> str:
    parts = []
    for j in range(idx, min(idx + 50, len(tokens))):
        if tokens[j].value == ';':
            break
        parts.append(tokens[j].value)
    return ''.join(parts)


def _text_until_brace(tokens: List[Token], idx: int) -> str:
    parts = []
    for j in range(idx, min(idx + 30, len(tokens))):
        if tokens[j].value == '{':
            break
        parts.append(tokens[j].value)
    return ''.join(parts)


def _text_until_bracket_close(tokens: List[Token], idx: int) -> str:
    parts = []
    for j in range(idx, min(idx + 20, len(tokens))):
        parts.append(tokens[j].value)
        if tokens[j].value == ']':
            break
    return ''.join(parts)


# ════════════════════════════════════════════════════════════════
#  TRANSFORMERS
# ════════════════════════════════════════════════════════════════

class OriginalTransformer:
    """Identity transform — no changes."""

    def transform(self, tokens: List[Token], mapping: Dict, **kw) -> str:
        return ''.join(t.value for t in tokens)


class MinimalTransformer:
    """Strip comments, rename identifiers to shortest names."""

    def transform(self, tokens: List[Token], mapping: Dict[str, str],
                  module_names: Set[str] = frozenset(), **kw) -> str:
        out: List[str] = []
        prev_was_newline = False
        for i, tok in enumerate(tokens):
            # Strip all comments
            if tok.type in (TT.LINE_COMMENT, TT.DOC_COMMENT, TT.BLOCK_COMMENT):
                continue

            # Collapse multiple blank lines (cosmetic)
            if tok.type == TT.WHITESPACE:
                # Keep structure but reduce consecutive blank lines
                lines = tok.value.split('\n')
                if len(lines) > 3:
                    out.append('\n\n')
                else:
                    out.append(tok.value)
                continue

            # Rename identifiers
            if tok.type == TT.IDENT:
                v = tok.value
                if v in PROTECTED_IDENTS or v in module_names:
                    out.append(v)
                elif v in mapping:
                    out.append(mapping[v])
                else:
                    out.append(v)
                continue

            out.append(tok.value)

        return ''.join(out)


class EducationalTransformer:
    """Expand names, add educational annotations."""

    def __init__(self, lang: str = 'en'):
        self.annotations = ANNOTATIONS_EN if lang == 'en' else ANNOTATIONS_FR

    def transform(self, tokens: List[Token], mapping: Dict[str, str],
                  module_names: Set[str] = frozenset(), **kw) -> str:
        # Detect annotation insertion points
        ann_points = detect_annotation_points(tokens)
        ann_map: Dict[int, str] = {}
        for idx, key in ann_points:
            if key in self.annotations:
                ann_map[idx] = self.annotations[key]

        out: List[str] = []
        for i, tok in enumerate(tokens):
            # Insert annotation before this token
            if i in ann_map:
                # Find current indentation
                indent = self._current_indent(tokens, i)
                out.append(f"{indent}{ann_map[i]}\n")

            # Rename identifiers (expand)
            if tok.type == TT.IDENT:
                v = tok.value
                if v in PROTECTED_IDENTS or v in module_names:
                    out.append(v)
                elif v in mapping:
                    out.append(mapping[v])
                else:
                    out.append(v)
                continue

            out.append(tok.value)

        return ''.join(out)

    @staticmethod
    def _current_indent(tokens: List[Token], idx: int) -> str:
        """Find the whitespace indentation for the current line."""
        for j in range(idx - 1, max(idx - 5, -1), -1):
            if tokens[j].type == TT.WHITESPACE and '\n' in tokens[j].value:
                after_nl = tokens[j].value.rsplit('\n', 1)[-1]
                return after_nl
        return ''


# ════════════════════════════════════════════════════════════════
#  MAIN TRANSLATOR
# ════════════════════════════════════════════════════════════════

class SourceTranslator:
    """
    Orchestrates the two-pass translation:
      Pass 1: tokenize all files, collect identifiers, build global rename mapping.
      Pass 2: apply transformation preset to all files and write output.
    """

    RUST_EXTENSIONS = {'.rs'}

    def __init__(self, preset: str, lang: str = 'en',
                 load_mapping: Optional[str] = None,
                 save_mapping: Optional[str] = None,
                 dry_run: bool = False,
                 verbose: bool = False):
        self.preset = preset
        self.lang = lang
        self.load_mapping_path = load_mapping
        self.save_mapping_path = save_mapping
        self.dry_run = dry_run
        self.verbose = verbose

        if preset == 'original':
            self.transformer = OriginalTransformer()
        elif preset == 'minimal':
            self.transformer = MinimalTransformer()
        elif preset == 'educational':
            self.transformer = EducationalTransformer(lang=lang)
        else:
            raise ValueError(f"Unknown preset: {preset}")

    def process(self, input_path: str, output_path: Optional[str] = None):
        input_p = Path(input_path).resolve()

        if input_p.is_file():
            files = [input_p]
            base_dir = input_p.parent
        elif input_p.is_dir():
            files = sorted(input_p.rglob('*.rs'))
            base_dir = input_p
        else:
            print(f"Error: {input_path} not found.", file=sys.stderr)
            sys.exit(1)

        if not files:
            print("No .rs files found.", file=sys.stderr)
            sys.exit(1)

        print(f"[source_translator] Preset: {self.preset}"
              + (f" ({self.lang})" if self.preset == 'educational' else ''))
        print(f"[source_translator] Found {len(files)} Rust file(s)")

        # ── Pass 1: tokenize & collect ──
        all_tokens: Dict[str, List[Token]] = {}
        ident_counts: Counter = Counter()

        for fpath in files:
            if self.verbose:
                print(f"  Tokenizing {fpath.relative_to(base_dir)}")
            try:
                source = fpath.read_text(encoding='utf-8', errors='replace')
            except Exception as e:
                print(f"  Warning: could not read {fpath}: {e}", file=sys.stderr)
                continue
            tokenizer = RustTokenizer(source)
            tokens = tokenizer.tokenize()
            key = str(fpath)
            all_tokens[key] = tokens

            for tok in tokens:
                if tok.type == TT.IDENT and tok.value not in PROTECTED_IDENTS:
                    ident_counts[tok.value] += 1

        # Detect module names (protected from renaming)
        module_names = collect_module_names(all_tokens)
        # Remove module names from renameable set
        for mn in module_names:
            if mn in ident_counts:
                del ident_counts[mn]

        # ── Build or load mapping ──
        if self.load_mapping_path:
            with open(self.load_mapping_path, 'r', encoding='utf-8') as f:
                mapping = json.load(f)
            print(f"[source_translator] Loaded mapping with {len(mapping)} entries")
        else:
            mapping = self._build_mapping(ident_counts)
            print(f"[source_translator] Built mapping: {len(mapping)} identifiers")

        # ── Save mapping if requested ──
        if self.save_mapping_path:
            with open(self.save_mapping_path, 'w', encoding='utf-8') as f:
                json.dump(mapping, f, indent=2, ensure_ascii=False, sort_keys=True)
            print(f"[source_translator] Saved mapping to {self.save_mapping_path}")

        if self.dry_run:
            self._print_stats(ident_counts, mapping)
            return

        # ── Pass 2: transform & write ──
        if output_path is None:
            print("Error: --output is required (unless --dry-run).", file=sys.stderr)
            sys.exit(1)

        out_base = Path(output_path).resolve()
        written = 0

        for fpath in files:
            key = str(fpath)
            if key not in all_tokens:
                continue
            rel = fpath.relative_to(base_dir)
            dest = out_base / rel
            dest.parent.mkdir(parents=True, exist_ok=True)

            result = self.transformer.transform(
                all_tokens[key], mapping, module_names=module_names)
            dest.write_text(result, encoding='utf-8')
            written += 1
            if self.verbose:
                print(f"  Wrote {rel}")

        # Copy non-Rust files if processing a directory
        if input_p.is_dir():
            for other in input_p.rglob('*'):
                if other.is_file() and other.suffix not in self.RUST_EXTENSIONS:
                    rel = other.relative_to(base_dir)
                    dest = out_base / rel
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(other, dest)

        print(f"[source_translator] Done -- {written} file(s) written to {out_base}")
        self._print_stats(ident_counts, mapping)

    # ── mapping builder ─────────────────────────────────────

    def _build_mapping(self, ident_counts: Counter) -> Dict[str, str]:
        if self.preset == 'original':
            return {}

        # Sort by frequency (most used first → shortest name in minimal mode)
        sorted_idents = sorted(ident_counts.items(), key=lambda x: (-x[1], x[0]))

        if self.preset == 'minimal':
            gen = MinimalNameGen()
            mapping: Dict[str, str] = {}
            for name, _ in sorted_idents:
                style = _classify_ident(name)
                mapping[name] = gen.next_name(style)
            return mapping

        elif self.preset == 'educational':
            mapping: Dict[str, str] = {}
            for name, _ in sorted_idents:
                expanded = expand_identifier(name)
                # Only map if actually different
                if expanded != name:
                    mapping[name] = expanded
            return mapping

        return {}

    # ── stats ───────────────────────────────────────────────

    def _print_stats(self, ident_counts: Counter, mapping: Dict):
        total_idents = sum(ident_counts.values())
        unique = len(ident_counts)
        renamed = sum(1 for k in ident_counts if k in mapping)
        print(f"\n  Statistics:")
        print(f"    Total identifier occurrences : {total_idents}")
        print(f"    Unique identifiers           : {unique}")
        print(f"    Renamed                      : {renamed}")
        if self.preset == 'minimal' and mapping:
            original_bytes = sum(len(k) * v for k, v in ident_counts.items() if k in mapping)
            new_bytes = sum(len(mapping[k]) * v for k, v in ident_counts.items() if k in mapping)
            saved = original_bytes - new_bytes
            print(f"    Identifier bytes saved       : {saved:,} ({saved*100//max(original_bytes,1)}%)")
        print()
        # Show top 10 renames
        top = [(k, mapping.get(k, k)) for k, _ in sorted(ident_counts.items(), key=lambda x: -x[1])[:10]]
        print(f"  Top renames (by frequency):")
        for orig, new_name in top:
            tag = f"  ->  {new_name}" if orig != new_name else "  (unchanged)"
            print(f"    {orig:30s}{tag}")
        print()


# ════════════════════════════════════════════════════════════════
#  CLI
# ════════════════════════════════════════════════════════════════

def main():
    parser = argparse.ArgumentParser(
        prog='source_translator',
        description='TrustOS Source Code Translator — transform Rust code with presets.',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --preset minimal -i kernel/src/ -o translated/minimal/
  %(prog)s --preset educational --lang fr -i kernel/src/ -o translated/edu-fr/
  %(prog)s --preset minimal -i kernel/src/ --dry-run --save-mapping map.json
  %(prog)s --preset minimal -i kernel/src/ -o out/ --load-mapping map.json
        """)

    parser.add_argument('--preset', '-p', required=True,
                        choices=['original', 'minimal', 'educational'],
                        help='Transformation preset')
    parser.add_argument('--lang', '-l', default='en', choices=['en', 'fr'],
                        help='Language for educational annotations (default: en)')
    parser.add_argument('--input', '-i', required=True,
                        help='Input file or directory')
    parser.add_argument('--output', '-o', default=None,
                        help='Output directory (required unless --dry-run)')
    parser.add_argument('--save-mapping', default=None,
                        help='Save identifier mapping to JSON file')
    parser.add_argument('--load-mapping', default=None,
                        help='Load identifier mapping from JSON file')
    parser.add_argument('--dry-run', action='store_true',
                        help='Analyze and show stats without writing files')
    parser.add_argument('--verbose', '-v', action='store_true',
                        help='Show per-file progress')

    args = parser.parse_args()

    translator = SourceTranslator(
        preset=args.preset,
        lang=args.lang,
        load_mapping=args.load_mapping,
        save_mapping=args.save_mapping,
        dry_run=args.dry_run,
        verbose=args.verbose,
    )

    translator.process(args.input, args.output)


if __name__ == '__main__':
    main()
