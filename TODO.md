# RISC-V Emulator WASM JIT 実装計画

## 📋 Phase 1: WASM JIT基盤 (実装開始)
- [ ] `wasm/Cargo.toml` → `wasm-bindgen-futures = "0.4"` 追加
- [ ] `src/jit/mod.rs` **新規作成** (モジュール構造)
- [ ] `src/jit/wasm.rs` **新規作成** (WASMバイトコード生成)
- [ ] `wasm/src/lib.rs` → JIT API追加 (`enable_jit()`, `compile_trace()`)
- [ ] `wasm/web/App.js` → WASMコンパイル処理追加
- [ ] `src/cpu.rs` → `tick()` にJIT統合
- [ ] `src/lib.rs` → JIT設定API追加

## 📋 Phase 2: トレース記録・実行  
- [ ] ホットトレース検出 (実行カウンタ > 10000)
- [ ] 簡易整数命令のWASMバイトコード生成 (ADD, ADDI, SUB, LW, SW)
- [ ] トレース出口処理 (インタプリタfallback)
- [ ] wasm-bindgen-futuresを使ったasyncコンパイル

## 📋 Phase 3: 完全統合・テスト
- [ ] JS側の`run_cycles()`をJIT優先に変更
- [ ] 性能計測・デバッグ機能追加
- [ ] Linuxブートテスト
- [ ] riscv-tests テストスイート実行

## 📋 Phase 4: 拡張機能
- [ ] ロード/ストア・分岐命令対応
- [ ] 圧縮命令対応
- [ ] トレース連結最適化
- [ ] FPU命令対応

## 🎯 期待性能
```
整数ループ: 8-15x ↑  
メモリ: 4-10x ↑
全体: 3-8x ↑ (ブラウザ環境)
```

## 🧪 検証手順
```
1. cargo build --target wasm32-unknown-unknown
2. wasm-pack build wasm/ --target web
3. wasm/web/index.html 確認
4. Chrome DevTools 性能計測
5. Linuxブート時間比較
```

## 📊 成功基準
- [ ] Phase 1完了: `enable_jit()`動作
- [ ] Phase 2完了: 整数命令JIT化
- [ ] Phase 3完了: Linuxブート3x↑
- [ ] Phase 4完了: 8x↑達成

---
**ステータス**: 🟡 計画作成完了 | 実装待ち
**最終目標**: ブラウザでLinuxブートを高速化
