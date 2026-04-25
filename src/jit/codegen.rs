/// Memory access width for code generation
#[derive(Debug, Clone, Copy)]
pub enum MemWidth {
    B1,  // 1 byte
    B2,  // 2 bytes
    B4,  // 4 bytes
    B8,  // 8 bytes
}

/// JavaScript code generator for RISC-V instructions
pub struct CodeGenerator {
    /// Whether to use minified identifiers (for release builds)
    minify: bool,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new() -> Self {
        CodeGenerator { minify: false }
    }

    /// Set whether to minify identifiers
    pub fn set_minify(&mut self, minify: bool) {
        self.minify = minify;
    }

    /// Generate JavaScript code for a basic block starting at PC
    pub fn generate_block(&self, pc: u64, instructions: &[u8]) -> String {
        let max64 = if self.minify { "f" } else { "$.f" };
        let max32 = if self.minify { "g" } else { "0xffff_ffffn" };
        let signed = if self.minify { "s" } else { "(a=>BigInt.asIntN(64,a))" };
        let unsigned = if self.minify { "u" } else { "(a=>BigInt.asUintN(64,a))" };
        let data = if self.minify { "d" } else { "data" };

        let expected = u32::from_le_bytes([
            instructions[0],
            instructions[1],
            instructions[2],
            instructions[3],
        ]);

        // Build the JavaScript code step by step to avoid format string issues
        let mut code = String::new();

        code.push_str("return async function(){");
        code.push_str(&format!("let {}=$.f,", max64));
        code.push_str(&format!("{}=0xffff_ffffn,", max32));
        code.push_str(&format!("{}=(a=>BigInt.asIntN(64,a)),", signed));
        code.push_str(&format!("{}=(a=>BigInt.asUintN(64,a)),", unsigned));
        code.push_str(&format!("{}=(p=>{{", data));
        code.push_str("p=$.get_page(p);");
        code.push_str("return new DataView($._sys(`memory`).buffer,p);");
        code.push_str("}});");
        code.push_str(&format!("x{}:for(;;){{", pc));
        code.push_str(&format!("const p={}n;", pc));
        code.push_str(&format!("if({}(p).getUint32(0,true)!=={:#x}){{", data, expected));
        code.push_str(&format!("delete $.p[`x{}`];", pc));
        code.push_str("return J(p);");
        code.push_str("}}");
        code.push_str("// Instruction body placeholder");
        code.push_str("}}");
        code.push_str("}");

        code
    }

    /// Generate JavaScript code for loading a register
    pub fn generate_load_reg(&self, reg: u8) -> String {
        if reg == 0 {
            "0n".to_string()
        } else if self.minify {
            format!("(($._r??=$.r)[`x{}`]??=0n)", reg)
        } else {
            format!("(($._r??=$.r)[`x{}`]??=0n)", reg)
        }
    }

    /// Generate JavaScript code for storing a register
    pub fn generate_store_reg(&self, reg: u8, value_expr: &str) -> String {
        if reg == 0 {
            format!("{};", value_expr)
        } else if self.minify {
            format!("(($._r??=$.r)[`x{}`]={});", reg, value_expr)
        } else {
            format!("(($._r??=$.r)[`x{}`]={});", reg, value_expr)
        }
    }

    /// Generate JavaScript code for arithmetic operations
    pub fn generate_arithmetic(&self, op: &str, left: &str, right: &str) -> String {
        let max64 = if self.minify { "f" } else { "$.f" };
        format!("({}{}{})&{}", left, op, right, max64)
    }

    /// Generate JavaScript code for memory load
    pub fn generate_load_mem(&self, addr_expr: &str, width: MemWidth, signed: bool) -> String {
        let data = if self.minify { "d" } else { "data" };
        let max64 = if self.minify { "f" } else { "$.f" };
        let max32 = if self.minify { "g" } else { "0xffff_ffffn" };
        let signed_fn = if self.minify { "s" } else { "BigInt.asIntN(64,a)" };
        let unsigned_fn = if self.minify { "u" } else { "BigInt.asUintN(64,a)" };

        match (width, signed) {
            (MemWidth::B1, true) => {
                format!("{}({}({}({})&{}).getInt8(0,true)))", unsigned_fn, signed_fn, data, addr_expr, max64)
            }
            (MemWidth::B1, false) => {
                format!("BigInt({}({})&{}).getUint8(0,true))", data, addr_expr, max64)
            }
            (MemWidth::B2, true) => {
                format!("{}({}({}({})&{}).getInt16(0,true)))", unsigned_fn, signed_fn, data, addr_expr, max64)
            }
            (MemWidth::B2, false) => {
                format!("BigInt({}({})&{}).getUint16(0,true))", data, addr_expr, max64)
            }
            (MemWidth::B4, true) => {
                format!("{}({}({}({})&{}).getInt32(0,true)))", unsigned_fn, signed_fn, data, addr_expr, max64)
            }
            (MemWidth::B4, false) => {
                format!("BigInt({}({})&{}).getUint32(0,true))", data, addr_expr, max64)
            }
            (MemWidth::B8, _) => {
                format!("{}({})&{}).getBigUint64(0,true)", data, addr_expr, max64)
            }
        }
    }

    /// Generate JavaScript code for memory store
    pub fn generate_store_mem(&self, addr_expr: &str, value_expr: &str, width: MemWidth) -> String {
        let data = if self.minify { "d" } else { "data" };
        let max64 = if self.minify { "f" } else { "$.f" };
        let max32 = if self.minify { "g" } else { "0xffff_ffffn" };

        match width {
            MemWidth::B1 => {
                format!("{}({})&{}).setUint8(0,Number({}&{}),true);", data, addr_expr, max64, value_expr, max32)
            }
            MemWidth::B2 => {
                format!("{}({})&{}).setUint16(0,Number({}&{}),true);", data, addr_expr, max64, value_expr, max32)
            }
            MemWidth::B4 => {
                format!("{}({})&{}).setUint32(0,Number({}&{}),true);", data, addr_expr, max64, value_expr, max32)
            }
            MemWidth::B8 => {
                format!("{}({})&{}).setBigUint64(0,{},true);", data, addr_expr, max64, value_expr)
            }
        }
    }

    /// Generate JavaScript code for immediate value
    pub fn generate_imm(&self, imm: u64) -> String {
        format!("{}n", imm)
    }

    /// Generate JavaScript code for comparison
    pub fn generate_comparison(&self, op: &str, left: &str, right: &str) -> String {
        format!("({}{}{}?1n:0n)", left, op, right)
    }

    /// Generate JavaScript code for conditional branch
    pub fn generate_branch(&self, condition: &str, target_pc: u64, next_pc: u64) -> String {
        format!(
            "if({}!==0n){{return J({}n);}}return J({}n);",
            condition, target_pc, next_pc
        )
    }

    /// Generate JavaScript code for unconditional jump
    pub fn generate_jump(&self, target_pc: u64) -> String {
        format!("return J({}n);", target_pc)
    }

    /// Generate JavaScript code for trap/error
    pub fn generate_trap(&self, message: &str) -> String {
        format!("throw new TypeError(`{}`);", message)
    }

    /// Generate JavaScript code for ECALL
    pub fn generate_ecall(&self) -> String {
        "await $.ecall();".to_string()
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_creation() {
        let gen = CodeGenerator::new();
        assert!(!gen.minify);
    }

    #[test]
    fn test_minify() {
        let mut gen = CodeGenerator::new();
        assert!(!gen.minify);
        
        gen.set_minify(true);
        assert!(gen.minify);
    }

    #[test]
    fn test_load_reg() {
        let gen = CodeGenerator::new();
        
        // x0 should always be 0
        assert_eq!(gen.generate_load_reg(0), "0n");
        
        // Other registers should load from the register file
        let x1 = gen.generate_load_reg(1);
        assert!(x1.contains("x1"));
        assert!(x1.contains("$.r"));
    }

    #[test]
    fn test_store_reg() {
        let gen = CodeGenerator::new();
        
        // x0 should discard the value
        let x0 = gen.generate_store_reg(0, "42n");
        assert!(x0.contains("42n"));
        assert!(!x0.contains("x0"));
        
        // Other registers should store the value
        let x1 = gen.generate_store_reg(1, "42n");
        assert!(x1.contains("x1"));
        assert!(x1.contains("42n"));
    }

    #[test]
    fn test_arithmetic() {
        let gen = CodeGenerator::new();
        
        let add = gen.generate_arithmetic("+", "a", "b");
        assert!(add.contains("+"));
        assert!(add.contains("$.f"));
        
        let sub = gen.generate_arithmetic("-", "a", "b");
        assert!(sub.contains("-"));
    }

    #[test]
    fn test_imm() {
        let gen = CodeGenerator::new();
        
        assert_eq!(gen.generate_imm(42), "42n");
        assert_eq!(gen.generate_imm(0), "0n");
        assert_eq!(gen.generate_imm(0xFFFFFFFF), "4294967295n");
    }

    #[test]
    fn test_comparison() {
        let gen = CodeGenerator::new();
        
        let eq = gen.generate_comparison("===", "a", "b");
        assert!(eq.contains("==="));
        assert!(eq.contains("1n"));
        assert!(eq.contains("0n"));
    }

    #[test]
    fn test_branch() {
        let gen = CodeGenerator::new();
        
        let branch = gen.generate_branch("cond", 0x1000, 0x1004);
        assert!(branch.contains("0x1000n"));
        assert!(branch.contains("0x1004n"));
        assert!(branch.contains("J"));
    }

    #[test]
    fn test_jump() {
        let gen = CodeGenerator::new();
        
        let jump = gen.generate_jump(0x1000);
        assert!(jump.contains("0x1000n"));
        assert!(jump.contains("J"));
    }

    #[test]
    fn test_trap() {
        let gen = CodeGenerator::new();
        
        let trap = gen.generate_trap("test error");
        assert!(trap.contains("test error"));
        assert!(trap.contains("TypeError"));
    }

    #[test]
    fn test_ecall() {
        let gen = CodeGenerator::new();
        
        let ecall = gen.generate_ecall();
        assert!(ecall.contains("ecall"));
    }

    #[test]
    fn test_generate_block() {
        let gen = CodeGenerator::new();
        
        let instructions = [0x93, 0x00, 0x00, 0x00]; // ADDI x1, x0, 0
        let block = gen.generate_block(0x1000, &instructions);
        
        assert!(block.contains("async function"));
        assert!(block.contains("x1000"));
        assert!(block.contains("0x93000000"));
        assert!(block.contains("$.f"));
        assert!(block.contains("0xffff_ffffn"));
    }
}
