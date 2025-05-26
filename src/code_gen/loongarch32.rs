use crate::ast::{Program, Stmt, Expr, BinOp, Variable};
use crate::semantic_analyzer::SymbolTable;
use std::collections::HashMap;

/// LoongArch32精简版的代码生成器
pub struct LoongArch32Reduce {
    // 寄存器分配映射
    reg_map: HashMap<String, String>,
    // 符号表
    symbol_table: SymbolTable,
    // 当前寄存器使用情况
    available_regs: Vec<String>,
    // 当前标签计数
    label_count: usize,
    // 当前栈空间大小
    stack_size: usize,
    // 生成的汇编代码
    assembly: Vec<String>,
}

impl LoongArch32Reduce {
    pub fn new(symbol_table: SymbolTable) -> Self {
        // 初始化可用寄存器列表 (根据LoongArch32精简版寄存器规范)
        let available_regs = vec![
            "$t0".to_string(), "$t1".to_string(), "$t2".to_string(),
            "$t3".to_string(), "$t4".to_string(), "$t5".to_string(),
            "$t6".to_string(), "$t7".to_string(), "$t8".to_string(),
        ];

        LoongArch32Reduce {
            reg_map: HashMap::new(),
            symbol_table,
            available_regs,
            label_count: 0,
            stack_size: 0,
            assembly: Vec::new(),
        }
    }

    /// 生成程序的汇编代码
    pub fn generate_code(&mut self, program: &Program) -> String {
        self.assembly.clear();
        
        // 添加程序头
        self.emit(".text");
        self.emit(".globl main");
        self.emit("main:");
        
        // 生成栈帧
        self.emit("    addi.w $sp, $sp, -128    # 分配栈空间");
        self.emit("    st.w $ra, $sp, 124       # 保存返回地址");
        self.emit("    st.w $fp, $sp, 120       # 保存帧指针");
        self.emit("    addi.w $fp, $sp, 128     # 设置新的帧指针");

        // 根据程序类型生成代码
        match program {
            Program::Full { body, declare, procs, .. } => {
                // 处理全局变量
                for var_dec in &declare.var_decs {
                    for name in &var_dec.names {
                        // 为变量分配栈空间
                        self.stack_size += 4; // 假设所有变量都是4字节
                        let offset = self.stack_size;
                        self.emit(&format!("    addi.w $t0, $fp, -{}    # 变量 {}", offset, name));
                        self.reg_map.insert(name.clone(), format!("-{}", offset));
                    }
                }

                // 处理程序体语句
                for stmt in &body.stmts {
                    self.generate_statement(stmt);
                }

                // 处理过程定义 (函数)
                for proc in procs {
                    self.emit(&format!("{}:", proc.name));
                    // 函数序言
                    self.emit("    addi.w $sp, $sp, -128    # 分配栈空间");
                    self.emit("    st.w $ra, $sp, 124       # 保存返回地址");
                    self.emit("    st.w $fp, $sp, 120       # 保存帧指针");
                    self.emit("    addi.w $fp, $sp, 128     # 设置新的帧指针");
                    
                    // 处理参数
                    let mut param_offset = 8; // 前两个参数可能在寄存器中，其他在栈上
                    for param in &proc.params {
                        for name in &param.names {
                            self.reg_map.insert(name.clone(), format!("{}", param_offset));
                            param_offset += 4;
                        }
                    }

                    // 处理函数体
                    for stmt in &proc.body.stmts {
                        self.generate_statement(stmt);
                    }

                    // 函数尾声
                    self.emit("    ld.w $ra, $sp, 124       # 恢复返回地址");
                    self.emit("    ld.w $fp, $sp, 120       # 恢复帧指针");
                    self.emit("    addi.w $sp, $sp, 128     # 释放栈空间");
                    self.emit("    jirl $zero, $ra, 0       # 返回");
                }
            }
        }

        // 添加程序尾
        self.emit("    ld.w $ra, $sp, 124       # 恢复返回地址");
        self.emit("    ld.w $fp, $sp, 120       # 恢复帧指针");
        self.emit("    addi.w $sp, $sp, 128     # 释放栈空间");
        self.emit("    jr $ra                   # 返回");

        // 合并生成的汇编代码
        self.assembly.join("\n")
    }

    /// 生成语句的汇编代码
    fn generate_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { var, expr } => {
                // 计算表达式的值
                let result_reg = self.generate_expression(expr);
                
                // 存储到变量
                match var {
                    Variable::Simple(name) => {
                        if let Some(offset) = self.reg_map.get(name) {
                            // 如果是相对于$fp的负偏移，需要特殊处理
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap();
                                self.emit(&format!("    st.w {}, $fp, -{}    # {} = expr", result_reg, offset_val, name));
                            } else {
                                self.emit(&format!("    st.w {}, $fp, {}    # {} = expr", result_reg, offset, name));
                            }
                        }
                    },
                    Variable::Array(name, index_expr) => {
                        // 计算数组索引
                        let index_reg = self.generate_expression(index_expr);
                        let base_reg = "$t8";
                        
                        if let Some(offset) = self.reg_map.get(name) {
                            // 计算数组基址
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap();
                                self.emit(&format!("    addi.w {}, $fp, -{}    # 数组基址", base_reg, offset_val));
                            } else {
                                self.emit(&format!("    addi.w {}, $fp, {}    # 数组基址", base_reg, offset));
                            }
                            
                            // 计算元素地址: base + index * 4
                            self.emit(&format!("    slli.w $t9, {}, 2        # index * 4", index_reg));
                            self.emit(&format!("    add.w $t9, {}, $t9       # 数组元素地址", base_reg));
                            self.emit(&format!("    st.w {}, $t9, 0          # 存储到数组元素", result_reg));
                        }
                    },
                    Variable::Record(name, field) => {
                        // 简单处理记录，假设字段有固定偏移
                        if let Some(offset) = self.reg_map.get(name) {
                            let field_offset = 4; // 假设每个字段占4字节
                            
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap() + field_offset;
                                self.emit(&format!("    st.w {}, $fp, -{}    # {}.{} = expr", result_reg, offset_val, name, field));
                            } else {
                                let offset_val = offset.parse::<i32>().unwrap() + field_offset;
                                self.emit(&format!("    st.w {}, $fp, {}    # {}.{} = expr", result_reg, offset_val, name, field));
                            }
                        }
                    }
                }
                
                // 释放寄存器
                self.free_register(&result_reg);
            },
            
            Stmt::If { cond, then, els } => {
                // 生成标签
                let else_label = self.generate_label("else");
                let end_label = self.generate_label("endif");
                
                // 生成条件表达式
                let cond_reg = self.generate_expression(cond);
                
                // 条件判断，如果为0跳转到else
                self.emit(&format!("    beq {}, $zero, {}    # if条件判断", cond_reg, else_label));
                self.free_register(&cond_reg);
                
                // 生成then部分
                for stmt in then {
                    self.generate_statement(stmt);
                }
                
                // 跳过else部分
                self.emit(&format!("    b {}          # 跳过else", end_label));
                
                // else部分
                self.emit(&format!("{}:", else_label));
                for stmt in els {
                    self.generate_statement(stmt);
                }
                
                // if结束
                self.emit(&format!("{}:", end_label));
            },
            
            Stmt::While { cond, body } => {
                // 生成标签
                let start_label = self.generate_label("while");
                let end_label = self.generate_label("endwhile");
                
                // while循环开始
                self.emit(&format!("{}:", start_label));
                
                // 生成条件表达式
                let cond_reg = self.generate_expression(cond);
                
                // 条件判断，如果为0跳出循环
                self.emit(&format!("    beq {}, $zero, {}    # while条件判断", cond_reg, end_label));
                self.free_register(&cond_reg);
                
                // 生成循环体
                for stmt in body {
                    self.generate_statement(stmt);
                }
                
                // 跳回循环开始
                self.emit(&format!("    b {}          # 返回while开始", start_label));
                
                // while结束
                self.emit(&format!("{}:", end_label));
            },

            Stmt::Read(var) => {
                // 简单实现：调用系统函数读取整数
                self.emit("    ori $a7, $zero, 5     # 系统调用号: 读取整数");
                self.emit("    syscall 0             # 进行系统调用");
                
                match var {
                    Variable::Simple(name) => {
                        if let Some(offset) = self.reg_map.get(name) {
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap();
                                self.emit(&format!("    st.w $a0, $fp, -{}    # 存储读取的值到 {}", offset_val, name));
                            } else {
                                self.emit(&format!("    st.w $a0, $fp, {}    # 存储读取的值到 {}", offset, name));
                            }
                        }
                    },
                    Variable::Array(name, index_expr) => {
                        // 计算数组索引
                        let index_reg = self.generate_expression(index_expr);
                        let base_reg = "$t8";
                        
                        if let Some(offset) = self.reg_map.get(name) {
                            // 计算数组基址
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap();
                                self.emit(&format!("    addi.w {}, $fp, -{}    # 数组基址", base_reg, offset_val));
                            } else {
                                self.emit(&format!("    addi.w {}, $fp, {}    # 数组基址", base_reg, offset));
                            }
                            
                            // 计算元素地址: base + index * 4
                            self.emit(&format!("    slli.w $t9, {}, 2        # index * 4", index_reg));
                            self.emit(&format!("    add.w $t9, {}, $t9       # 数组元素地址", base_reg));
                            self.emit(&format!("    st.w $a0, $t9, 0         # 存储读取的值到数组元素"));
                        }
                        
                        self.free_register(&index_reg);
                    },
                    Variable::Record(name, field) => {
                        // 简单处理记录，假设字段有固定偏移
                        if let Some(offset) = self.reg_map.get(name) {
                            let field_offset = 4; // 假设每个字段占4字节
                            
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap() + field_offset;
                                self.emit(&format!("    st.w $a0, $fp, -{}    # 存储读取的值到 {}.{}", offset_val, name, field));
                            } else {
                                let offset_val = offset.parse::<i32>().unwrap() + field_offset;
                                self.emit(&format!("    st.w $a0, $fp, {}    # 存储读取的值到 {}.{}", offset_val, name, field));
                            }
                        }
                    }
                }
            },
            
            Stmt::Write(expr) => {
                // 计算表达式的值
                let result_reg = self.generate_expression(expr);
                
                // 输出
                self.emit(&format!("    ori $a0, {}, 0      # 加载要打印的值", result_reg));
                self.emit("    ori $a7, $zero, 1     # 系统调用号: 打印整数");
                self.emit("    syscall 0             # 进行系统调用");
                
                // 输出换行符
                self.emit("    ori $a0, $zero, 10    # 换行符(\\n)的ASCII码");
                self.emit("    ori $a7, $zero, 11    # 系统调用号: 打印字符");
                self.emit("    syscall 0             # 进行系统调用");
                
                self.free_register(&result_reg);
            },
            
            Stmt::Call { name, args } => {
                // 保存所有被调用者保存的寄存器(简化处理)
                self.emit("    addi.w $sp, $sp, -32    # 保存寄存器");
                self.emit("    st.w $t0, $sp, 0");
                self.emit("    st.w $t1, $sp, 4");
                self.emit("    st.w $t2, $sp, 8");
                self.emit("    st.w $t3, $sp, 12");
                
                // 计算并传递参数
                for (i, arg) in args.iter().enumerate() {
                    let arg_reg = self.generate_expression(arg);
                    if i < 4 { // 前4个参数通过寄存器传递
                        self.emit(&format!("    ori $a{}, {}, 0    # 参数 {}", i, arg_reg, i));
                    } else { // 剩余参数压栈
                        self.emit(&format!("    st.w {}, $sp, {}    # 参数 {}", arg_reg, (i-4)*4, i));
                    }
                    self.free_register(&arg_reg);
                }
                
                // 调用函数
                self.emit(&format!("    bl {}         # 调用函数", name));
                
                // 恢复寄存器
                self.emit("    ld.w $t0, $sp, 0");
                self.emit("    ld.w $t1, $sp, 4");
                self.emit("    ld.w $t2, $sp, 8");
                self.emit("    ld.w $t3, $sp, 12");
                self.emit("    addi.w $sp, $sp, 32    # 恢复栈指针");
            },

            Stmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let result_reg = self.generate_expression(expr);
                    self.emit(&format!("    ori $a0, {}, 0      # 设置返回值", result_reg));
                    self.free_register(&result_reg);
                }
                
                // 生成函数返回代码
                self.emit("    ld.w $ra, $sp, 124    # 恢复返回地址");
                self.emit("    ld.w $fp, $sp, 120    # 恢复帧指针");
                self.emit("    addi.w $sp, $sp, 128  # 释放栈帧");
                self.emit("    jirl $zero, $ra, 0    # 返回");
            },
        }
    }

    /// 生成表达式的汇编代码，返回存放结果的寄存器
    fn generate_expression(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Int(value) => {
                let result_reg = self.allocate_register();
                self.emit(&format!("    ori {}, $zero, {}    # 加载立即数", result_reg, value));
                result_reg
            },
            
            Expr::Var(var) => {
                let result_reg = self.allocate_register();
                
                match var {
                    Variable::Simple(name) => {
                        if let Some(offset) = self.reg_map.get(name) {
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap();
                                self.emit(&format!("    ld.w {}, $fp, -{}    # 加载变量 {}", result_reg, offset_val, name));
                            } else {
                                self.emit(&format!("    ld.w {}, $fp, {}    # 加载变量 {}", result_reg, offset, name));
                            }
                        }
                    },
                    Variable::Array(name, index_expr) => {
                        // 计算数组索引
                        let index_reg = self.generate_expression(index_expr);
                        let base_reg = "$t8";
                        
                        if let Some(offset) = self.reg_map.get(name) {
                            // 计算数组基址
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap();
                                self.emit(&format!("    addi.w {}, $fp, -{}    # 数组基址", base_reg, offset_val));
                            } else {
                                self.emit(&format!("    addi.w {}, $fp, {}    # 数组基址", base_reg, offset));
                            }
                            
                            // 计算元素地址: base + index * 4
                            self.emit(&format!("    slli.w $t9, {}, 2        # index * 4", index_reg));
                            self.emit(&format!("    add.w $t9, {}, $t9       # 数组元素地址", base_reg));
                            self.emit(&format!("    ld.w {}, $t9, 0          # 加载数组元素", result_reg));
                        }
                        
                        self.free_register(&index_reg);
                    },
                    Variable::Record(name, field) => {
                        // 简单处理记录，假设字段有固定偏移
                        if let Some(offset) = self.reg_map.get(name) {
                            let field_offset = 4; // 假设每个字段占4字节
                            
                            if offset.starts_with('-') {
                                let offset_val = offset[1..].parse::<i32>().unwrap() + field_offset;
                                self.emit(&format!("    ld.w {}, $fp, -{}    # 加载 {}.{}", result_reg, offset_val, name, field));
                            } else {
                                let offset_val = offset.parse::<i32>().unwrap() + field_offset;
                                self.emit(&format!("    ld.w {}, $fp, {}    # 加载 {}.{}", result_reg, offset_val, name, field));
                            }
                        }
                    }
                }
                
                result_reg
            },
            
            Expr::Binary { left, op, right } => {
                // 计算左操作数
                let left_reg = self.generate_expression(left);
                
                // 计算右操作数
                let right_reg = self.generate_expression(right);
                
                // 根据操作符生成相应的指令
                match op {
                    BinOp::Add => {
                        self.emit(&format!("    add.w {}, {}, {}    # 加法", left_reg, left_reg, right_reg));
                    },
                    BinOp::Sub => {
                        self.emit(&format!("    sub.w {}, {}, {}    # 减法", left_reg, left_reg, right_reg));
                    },
                    BinOp::Mul => {
                        self.emit(&format!("    mul.w {}, {}, {}    # 乘法", left_reg, left_reg, right_reg));
                    },
                    BinOp::Div => {
                        self.emit(&format!("    div.w {}, {}, {}    # 除法", left_reg, left_reg, right_reg));
                    },
                    BinOp::Lt => {
                        self.emit(&format!("    slt {}, {}, {}     # 小于比较", left_reg, left_reg, right_reg));
                    },
                    BinOp::Eq => {
                        // 相等比较需要两条指令
                        self.emit(&format!("    xor {}, {}, {}     # 比较相等(异或)", left_reg, left_reg, right_reg));
                        self.emit(&format!("    sltui {}, {}, 1     # 结果为0则相等", left_reg, left_reg));
                    },
                }
                
                // 释放右操作数的寄存器
                self.free_register(&right_reg);
                
                // 返回结果寄存器(复用左操作数的寄存器)
                left_reg
            },
            
            Expr::Paren(inner) => {
                // 括号表达式直接计算内部表达式
                self.generate_expression(inner)
            },
        }
    }

    /// 分配寄存器
    fn allocate_register(&mut self) -> String {
        if let Some(reg) = self.available_regs.pop() {
            reg
        } else {
            // 如果没有可用寄存器，则可以考虑将某些值溢出到栈上
            // 简化处理，返回一个固定的临时寄存器
            "$t0".to_string()
        }
    }

    /// 释放寄存器
    fn free_register(&mut self, reg: &str) {
        // 检查是否是可分配的寄存器
        if reg.starts_with("$t") && !self.available_regs.contains(&reg.to_string()) {
            self.available_regs.push(reg.to_string());
        }
    }

    /// 生成唯一标签
    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_count);
        self.label_count += 1;
        label
    }

    /// 添加汇编代码
    fn emit(&mut self, code: &str) {
        self.assembly.push(code.to_string());
    }
}
