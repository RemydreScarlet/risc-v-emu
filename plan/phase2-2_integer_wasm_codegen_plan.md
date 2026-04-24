# Phase 2-2 実装前プラン：簡易的な整数命令の WASM コード生成

## 1. 概要

Phase 2-2 の目標は、ホットトレース検出後に記録された RISC-V 整数命令（ADD, ADDI, SUB, LW, SW）を WebAssembly（WASM）バイトコードに変換し、JIT コンパイル可能にすることである。  
現在、`src/jit/wasm.rs` には WASM モジュール生成の基盤（`WasmBuilder`, `WasmInstruction` など）が存在するが、`RiscvToWasmTranslator::translate_instruction` は NOP を出力するだけのプレースホルダーである。本フェーズでは、実際の RISC-V 命令デコードと WASM 変換ロジックを実装する。

---

## 2. 情報収集結果

### 2.1 既存コード構造

| ファイル | 役割 | 関連箇所 |
|---------|------|---------|
| `src/jit/wasm.rs` | WASM バイトコード生成器 | `WasmBuilder`, `RiscvToWasmTranslator`, `WasmInstruction` 列挙型 |
| `src/jit/mod.rs` | JIT コンパイラ本体 | `JitCompiler`（トレースキャッシュ、実行カウンタ） |
| `src/cpu.rs` | RISC-V CPU エミュレータ | 命令実行ロジック、`parse_format_r`, `parse_format_i`, `parse_format_s` など |
| `src/lib.rs` | `Emulator` 統合 | JIT 有効化/無効化 API |
| `wasm/src/lib.rs` | wasm-bindgen インターフェース | `WasmRiscv::compile_trace()`（プレースホルダー） |

### 2.2 現状の課題

1. **命令デコードの重複**：`cpu.rs` のフォーマットパーサーは `cpu.rs` 内に閉じており、`wasm.rs` からは利用できない。JIT コンパイル時に再度デコードする必要がある。
2. **WASM 命令エンコーダの不足**：`write_instruction` 内で多くの命令（`I64And`, `I64Or`, `I64Shl`, `I64Load`, `I64Store`, `Call` など）が `UnsupportedInstruction` で落ちている。
3. **メモリアクセスモデル**：WASM の線形メモリと RISC-V の DRAM をどう対応させるか未定義。
4. **レジスタマッピング**：RISC-V x0-x31 を WASM のローカル変数にどうマッピングするか未定義。

---

## 3. 実装計画

### 3.1 全体アーキテクチャ

```
RISC-V Trace (Vec<u32>)
    ↓
[RISC-V Decoder] —— wasm.rs 内に新規実装
    ↓
WASM IR (Vec<WasmInstruction>)
    ↓
[WASM Encoder] —— WasmBuilder::generate_bytecode()
    ↓
WASM Bytecode (Vec<u8>)
    ↓
[JS Runtime] —— WebAssembly.instantiate()
    ↓
実行
```

### 3.2 レジスタマッピング

RISC-V 整数レジスタ x0-x31 を WASM 関数のローカル変数 `i64` として割り当てる。

| WASM Local | 用途 |
|-----------|------|
| 0 | x0 (zero) — 書き込みは無視してもよいが、読み出し用に確保 |
| 1-31 | x1-x31 |
| 32 | pc (トレース内で必要な場合) |
| 33+ | 一時変数 |

### 3.3 メモリアクセス戦略

WASM の線形メモリを RISC-V の物理メモリ全体にマッピングするのはページング・デバイスメモリの考慮が複雑となる。  
**Phase 2-2 では「ホスト関数呼び出し」方式を採用する**：

- `load_word(addr: i64) -> i64`（インポート関数）
- `store_word(addr: i64, value: i64)`（インポート関数）
- `load_doubleword(addr: i64) -> i64`
- `store_doubleword(addr: i64, value: i64)`

これにより、WASM コードはメモリアクセス時に単に `call` を発行し、実際の読み書きはホスト（Rust/JS）側のエミュレータメモリで行う。将来的に（Phase 4）WASM メモリ直接アクセスへの移行を検討する。

### 3.4 変換対象命令（Phase 2-2）

| RISC-V 命令 | WASM 命令列 | 備考 |
|------------|------------|------|
| `ADD rd, rs1, rs2` | `local.get rs1`, `local.get rs2`, `i64.add`, `local.set rd` | x0 書き込みはスキップ可能 |
| `ADDI rd, rs1, imm` | `local.get rs1`, `i64.const imm`, `i64.add`, `local.set rd` | imm は 12bit 符号拡張済みとする |
| `SUB rd, rs1, rs2` | `local.get rs1`, `local.get rs2`, `i64.sub`, `local.set rd` | |
| `LW rd, offset(rs1)` | `local.get rs1`, `i64.const offset`, `i64.add`, `call load_word`, `local.set rd` | 戻り値は符号拡張済みと仮定 |
| `SW rs2, offset(rs1)` | `local.get rs1`, `i64.const offset`, `i64.add`, `local.get rs2`, `call store_word` | |

---

## 4. 変更対象ファイルと詳細

### 4.1 `src/jit/wasm.rs`（主要変更）

#### A. RISC-V 命令デコーダー追加

```rust
// 新規追加
struct RiscvInstruction {
    opcode: u8,
    rd: u8,
    rs1: u8,
    rs2: u8,
    funct3: u8,
    funct7: u8,
    imm: i64,
}

fn decode_riscv_instruction(word: u32) -> RiscvInstruction { ... }
```

- `opcode`: `word & 0x7f`
- `rd`: `(word >> 7) & 0x1f`
- `rs1`: `(word >> 15) & 0x1f`
- `rs2`: `(word >> 20) & 0x1f`
- `funct3`: `(word >> 12) & 0x7`
- `funct7`: `(word >> 25) & 0x7f`
- `imm`: I-type / S-type などフォーマットに応じて計算

#### B. `RiscvToWasmTranslator` 拡張

```rust
pub struct RiscvToWasmTranslator {
    builder: WasmBuilder,
    current_function: Option<WasmFunction>,
    // 新規: インポート関数インデックスの管理
    load_word_index: Option<u32>,
    store_word_index: Option<u32>,
}
```

#### C. `translate_instruction` 実装

```rust
pub fn translate_instruction(&mut self, word: u32) -> Result<(), WasmGenerationError> {
    let inst = decode_riscv_instruction(word);
    match inst.opcode {
        0x33 => self.translate_r_type(inst),  // ADD, SUB, ...
        0x13 => self.translate_i_type(inst),  // ADDI, ...
        0x03 => self.translate_load(inst),    // LW, ...
        0x23 => self.translate_store(inst),   // SW, ...
        _ => Err(WasmGenerationError::UnsupportedInstruction),
    }
}
```

#### D. `write_instruction` の拡張

以下の WASM 命令エンコーディングを追加する：

- `I64And`, `I64Or`, `I64Xor`
- `I64Shl`, `I64ShrS`, `I64ShrU`
- `I64Load(align, offset)`, `I64Store(align, offset)`
- `I32Load(align, offset)`, `I32Store(align, offset)`
- `Call(index)`
- `If`, `Block`, `Loop`（制御フロー用、Phase 2-2 では未使用だがエンコーダ準備）
- `I64Eq`, `I64LtS`, ...（比較命令）

#### E. インポートセクションの生成

`WasmBuilder` にインポート関数登録 API を追加し、生成する WASM モジュールがホスト関数をインポートできるようにする。

```rust
pub fn add_import_function(&mut self, module: &str, name: &str, type_index: u32) -> u32;
```

### 4.2 `src/jit/mod.rs`（軽微な変更）

- `JitCompiler::mark_for_compilation` で、実際に `RiscvToWasmTranslator` を用いてコンパイルを実行する。
- トレース記録用のバッファ `Vec<u32>` を保持し、コンパイル閾値到達時に `start_trace` → `translate_instruction` × N → `finish_trace` の流れを実装。

```rust
fn mark_for_compilation(&mut self, addr: u64) {
    // 1. addr からの連続する命令をトレースバッファに記録（最大 N 命令）
    // 2. RiscvToWasmTranslator で変換
    // 3. CompiledTrace を生成してキャッシュ
}
```

**注意**: 実際のトレース記録には `cpu.rs` からの命令供給が必要だが、Phase 2-2 では「命令列を直接受け取って WASM 生成する」機能を優先し、トレース記録自体は簡易的なハードコード引数で検証する。

### 4.3 `wasm/src/lib.rs`（軽微な変更）

`compile_trace` を実際のトレースコンパイル呼び出しに置き換える。

```rust
pub fn compile_trace(&mut self, start_addr: u64, end_addr: u64) -> bool {
    // 1. start_addr..end_addr の範囲の命令をフェッチ
    // 2. RiscvToWasmTranslator でコンパイル
    // 3. wasm_bytes を JS 側に返す or 内部キャッシュ
    true
}
```

### 4.4 `src/cpu.rs`（最小限の変更）

- `tick()` 内の JIT 実行フックを有効化。コンパイル済みトレースがあれば、`execute_compiled_trace` を呼び出す（またはインタプリタ fallback）。
- **Phase 2-2 では完全な統合は行わず、フックポイントの準備に留める**（Phase 3 で統合）。

---

## 5. 実装ステップ

| ステップ | 内容 | 対象ファイル |
|---------|------|-----------|
| 1 | `write_instruction` の不足エンコーダを実装（`I64Load`, `I64Store`, `Call` など） | `src/jit/wasm.rs` |
| 2 | RISC-V 命令デコーダ `decode_riscv_instruction` を実装 | `src/jit/wasm.rs` |
| 3 | `RiscvToWasmTranslator` にレジスタマッピングとローカル変数セットアップを追加 | `src/jit/wasm.rs` |
| 4 | `translate_instruction` に ADD, ADDI, SUB の変換を実装 | `src/jit/wasm.rs` |
| 5 | `translate_instruction` に LW, SW の変換を実装（ホスト関数呼び出し方式） | `src/jit/wasm.rs` |
| 6 | `WasmBuilder` にインポート関数セクション生成を追加 | `src/jit/wasm.rs` |
| 7 | `JitCompiler` にトレースコンパイルパイプラインを接続 | `src/jit/mod.rs` |
| 8 | 単体テスト：簡易命令列の WASM バイトコード生成を検証 | `src/jit/wasm.rs` (test) |
| 9 | wasm-bindgen インターフェースを更新 | `wasm/src/lib.rs` |

---

## 6. テスト・検証計画

### 6.1 Rust 単体テスト

`src/jit/wasm.rs` に `#[cfg(test)]` を追加し、以下を検証する：

```rust
#[test]
fn test_translate_add() {
    let mut translator = RiscvToWasmTranslator::new();
    translator.start_trace(0x80000000);
    // ADD x3, x1, x2 => 0x002081b3
    translator.translate_instruction(0x002081b3).unwrap();
    let wasm_bytes = translator.finish_trace().unwrap();
    
    // WASM バイトコードが正しく生成されているか（マジックナンバー、バージョン、セクション構成）
    assert_eq!(&wasm_bytes[0..4], &[0x00, 0x61, 0x73, 0x6d]);
    // さらにデコードして命令列を検証（オプション）
}
```

同様に ADDI, SUB, LW, SW についても検証する。

### 6.2 統合テスト（手動）

1. `cargo build` でコンパイルエラーがないことを確認。
2. `wasm-pack build wasm/ --target web` で WASM ビルドが通ることを確認。
3. `wasm/web/index.html` で `jit compile 0x80000000 0x80000010` などのデバッグコマンドを実行し、WASM バイトコード生成がエラーなく完了することを確認。

---

## 7. リスクと注意点

| リスク | 対応策 |
|-------|--------|
| WASM 命令エンコーディングの誤り | マニュアル（Wasm spec）とテストで検証 |
| RISC-V 即値の符号拡張ミス | `cpu.rs` の実装と突き合わせて検証 |
| x0 (zero) レジスタへの書き込み | WASM 側では local.set 0 しても無視されないが、エミュレータ側で x0=0 を保証 |
| メモリアクセスの境界チェック | Phase 2-2 ではホスト関数側（Rust）で実施。WASM 側ではしない |
| 32bit/64bit 混在 | RV64I を前提とし、WASM の `i64` を使用。32bit 命令（ADDW など）は Phase 4 で対応 |

---

## 8. フォローアップ（Phase 3 以降）

- 分岐命令（BEQ, JAL など）の変換とトレース出口処理
- WASM メモリ直接アクセスへの移行（パフォーマンス向上）
- 圧縮命令対応
- `cpu.rs` との完全統合（自動的な JIT 実行フallback）
