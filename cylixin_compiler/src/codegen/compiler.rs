use std::collections::HashMap;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, BasicMetadataValueEnum};
use inkwell::basic_block::BasicBlock;
use inkwell::IntPredicate;
use crate::ast::*;

#[derive(Debug)]
pub enum CodegenError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    Unsupported(String),
    LLVMError(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UndefinedVariable(n) => write!(f, "Undefined variable: {}", n),
            CodegenError::UndefinedFunction(n) => write!(f, "Undefined function: {}", n),
            CodegenError::Unsupported(s) => write!(f, "Unsupported: {}", s),
            CodegenError::LLVMError(s) => write!(f, "LLVM error: {}", s),
        }
    }
}

struct LoopContext<'ctx> {
    exit_block: BasicBlock<'ctx>,
    label: Option<String>,
}

pub struct Compiler<'ctx> {
    pub(crate) context:   &'ctx Context,
    pub(crate) module:    Module<'ctx>,
    pub(crate) builder:   Builder<'ctx>,
    pub(crate) variables: HashMap<String, (PointerValue<'ctx>, CyType)>,
    pub(crate) functions: HashMap<String, (FunctionValue<'ctx>, Option<CyType>)>,
    loop_stack: Vec<LoopContext<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("cylixin");
        let builder = context.create_builder();
        Compiler {
            context, module, builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    pub fn compile(&mut self, program: &Program) -> Result<String, CodegenError> {
        self.declare_printf();
        self.declare_pow();

        // first pass: declare all functions so they can call each other
        for stmt in &program.body {
            if let Stmt::FunDecl { name, params, return_type, .. } = stmt {
                self.declare_function(name, params, return_type)?;
            }
        }

        // compile user functions
        for stmt in &program.body {
            if let Stmt::FunDecl { .. } = stmt {
                self.compile_stmt(stmt)?;
            }
        }

        // compile top-level code into main()
        let i64_type = self.context.i64_type();
        let main_type = i64_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        for stmt in &program.body {
            if let Stmt::FunDecl { .. } = stmt { continue; }
            self.compile_stmt(stmt)?;
        }

        let zero = i64_type.const_int(0, false);
        self.builder.build_return(Some(&zero))
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        Ok(self.module.print_to_string().to_string())
    }

    fn declare_printf(&self) {
        let i32_type = self.context.i32_type();
        let ptr_type = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        let printf_type = i32_type.fn_type(&[ptr_type.into()], true);
        self.module.add_function("printf", printf_type, Some(inkwell::module::Linkage::External));
    }

    fn declare_pow(&self) {
        let f64_type = self.context.f64_type();
        let pow_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
        self.module.add_function("pow", pow_type, Some(inkwell::module::Linkage::External));
    }

    fn declare_function(&mut self, name: &str, params: &[Param], return_type: &Option<CyType>)
        -> Result<(), CodegenError>
    {
        let param_types: Vec<BasicMetadataTypeEnum> = params.iter()
            .map(|p| self.cy_type_to_metadata(&p.type_ann))
            .collect();

        let fn_type = match return_type {
            Some(ty) => {
                let ret = self.cy_type_to_llvm(ty);
                ret.fn_type(&param_types, false)
            }
            None => self.context.void_type().fn_type(&param_types, false),
        };

        let func = self.module.add_function(name, fn_type, None);
        self.functions.insert(name.to_string(), (func, return_type.clone()));
        Ok(())
    }

    pub(crate) fn cy_type_to_llvm(&self, ty: &CyType) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            CyType::Int | CyType::Long => self.context.i64_type().into(),
            CyType::Float => self.context.f64_type().into(),
            CyType::Bool => self.context.bool_type().into(),
            CyType::Char => self.context.i8_type().into(),
            CyType::StringType => self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),
            _ => self.context.i64_type().into(),
        }
    }

    fn cy_type_to_metadata(&self, ty: &CyType) -> BasicMetadataTypeEnum<'ctx> {
        match ty {
            CyType::Int | CyType::Long => self.context.i64_type().into(),
            CyType::Float => self.context.f64_type().into(),
            CyType::Bool => self.context.bool_type().into(),
            CyType::Char => self.context.i8_type().into(),
            CyType::StringType => self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),
            _ => self.context.i64_type().into(),
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match stmt {
            Stmt::VarDecl { name, type_ann, initialiser, .. } => {
                let ty = type_ann.as_ref().unwrap_or(&CyType::Int);
                let llvm_ty = self.cy_type_to_llvm(ty);
                let ptr = self.builder.build_alloca(llvm_ty, name)
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                if let Some(init) = initialiser {
                    let (val, inferred_ty) = self.compile_expr(init)?;
                    self.builder.build_store(ptr, val)
                        .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                    // use the inferred type if no annotation
                    let actual_ty = if type_ann.is_some() { ty.clone() } else { inferred_ty };
                    self.variables.insert(name.clone(), (ptr, actual_ty));
                } else {
                    self.variables.insert(name.clone(), (ptr, ty.clone()));
                }
                Ok(())
            }
            Stmt::Assign { target, op, value } => {
                let (val, _) = self.compile_expr(value)?;
                match target {
                    AssignTarget::Ident(name) => {
                        let (ptr, ty) = self.variables.get(name)
                            .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))?;
                        let ptr = *ptr;
                        let ty = ty.clone();
                        let final_val = if matches!(op, AssignOp::Assign) {
                            val
                        } else {
                            let llvm_ty = self.cy_type_to_llvm(&ty);
                            let current = self.builder.build_load(llvm_ty, ptr, "cur")
                                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                            self.apply_compound_op(&ty, current, val, op)?
                        };
                        self.builder.build_store(ptr, final_val)
                            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                        Ok(())
                    }
                    AssignTarget::Index { .. } => {
                        Err(CodegenError::Unsupported("index assignment".into()))
                    }
                }
            }
            Stmt::ExprStmt(expr) => { self.compile_expr(expr)?; Ok(()) }
            Stmt::If { condition, then_body, elif_arms, else_body, end_when } => {
                self.compile_if(condition, then_body, elif_arms, else_body)?;
                self.compile_end_when(end_when)
            }
            Stmt::ForRange { var, from, to, body, label, end_when } => {
                self.compile_for_range(var, from, to, body, label)?;
                self.compile_end_when(end_when)
            }
            Stmt::ForC { init, cond, update, body, label, end_when } => {
                self.compile_for_c(init, cond, update, body, label)?;
                self.compile_end_when(end_when)
            }
            Stmt::While { condition, body, label, end_when } => {
                self.compile_while(condition, body, label)?;
                self.compile_end_when(end_when)
            }
            Stmt::FunDecl { name, params, return_type, body } => {
                self.compile_fun_decl(name, params, return_type, body)
            }
            Stmt::Return(expr) => self.compile_return(expr),
            Stmt::Break(label) => self.compile_break(label),
        }
    }

    fn apply_compound_op(&self, ty: &CyType, lhs: BasicValueEnum<'ctx>, rhs: BasicValueEnum<'ctx>, op: &AssignOp)
        -> Result<BasicValueEnum<'ctx>, CodegenError>
    {
        match (ty, op) {
            (CyType::Int | CyType::Long, AssignOp::AddAssign) => {
                let r = self.builder.build_int_add(lhs.into_int_value(), rhs.into_int_value(), "add")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(r.into())
            }
            (CyType::Int | CyType::Long, AssignOp::SubAssign) => {
                let r = self.builder.build_int_sub(lhs.into_int_value(), rhs.into_int_value(), "sub")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(r.into())
            }
            (CyType::Int | CyType::Long, AssignOp::MulAssign) => {
                let r = self.builder.build_int_mul(lhs.into_int_value(), rhs.into_int_value(), "mul")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(r.into())
            }
            (CyType::Int | CyType::Long, AssignOp::DivAssign) => {
                let r = self.builder.build_int_signed_div(lhs.into_int_value(), rhs.into_int_value(), "div")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(r.into())
            }
            (CyType::Float, AssignOp::AddAssign) => {
                let r = self.builder.build_float_add(lhs.into_float_value(), rhs.into_float_value(), "fadd")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(r.into())
            }
            (CyType::Float, AssignOp::SubAssign) => {
                let r = self.builder.build_float_sub(lhs.into_float_value(), rhs.into_float_value(), "fsub")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(r.into())
            }
            (CyType::Int | CyType::Long, AssignOp::ModAssign) => {
                let r = self.builder.build_int_signed_rem(lhs.into_int_value(), rhs.into_int_value(), "mod")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(r.into())
            }
            // **= calls C's pow() just like the binary ** operator
            (CyType::Int | CyType::Long, AssignOp::PowAssign) => {
                let pow_fn = self.module.get_function("pow")
                    .ok_or_else(|| CodegenError::UndefinedFunction("pow".into()))?;
                let f64_type = self.context.f64_type();
                let lf = self.builder.build_signed_int_to_float(lhs.into_int_value(), f64_type, "lf")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                let rf = self.builder.build_signed_int_to_float(rhs.into_int_value(), f64_type, "rf")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                let result = self.builder.build_call(pow_fn, &[lf.into(), rf.into()], "pow")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?
                    .try_as_basic_value();
                match result {
                    inkwell::values::ValueKind::Basic(v) => {
                        let i = self.builder.build_float_to_signed_int(v.into_float_value(), self.context.i64_type(), "ipow")
                            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                        Ok(i.into())
                    }
                    _ => Err(CodegenError::LLVMError("pow returned no value".into())),
                }
            }
            (CyType::Float, AssignOp::PowAssign) => {
                let pow_fn = self.module.get_function("pow")
                    .ok_or_else(|| CodegenError::UndefinedFunction("pow".into()))?;
                let result = self.builder.build_call(pow_fn, &[lhs.into(), rhs.into()], "fpow")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?
                    .try_as_basic_value();
                match result {
                    inkwell::values::ValueKind::Basic(v) => Ok(v),
                    _ => Err(CodegenError::LLVMError("pow returned no value".into())),
                }
            }
            _ => Err(CodegenError::Unsupported(format!("compound {:?} on {:?}", op, ty))),
        }
    }

    fn compile_if(&mut self, condition: &Expr, then_body: &[Stmt],
        elif_arms: &[ElifArm], else_body: &Option<Vec<Stmt>>)
        -> Result<(), CodegenError>
    {
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let then_bb = self.context.append_basic_block(parent, "then");
        let merge_bb = self.context.append_basic_block(parent, "endif");

        // figure out what comes after the then block
        let first_else = if !elif_arms.is_empty() {
            self.context.append_basic_block(parent, "elif0")
        } else if else_body.is_some() {
            self.context.append_basic_block(parent, "else")
        } else {
            merge_bb
        };

        let (cond_val, _) = self.compile_expr(condition)?;
        self.builder.build_conditional_branch(cond_val.into_int_value(), then_bb, first_else)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        // then
        self.builder.position_at_end(then_bb);
        for s in then_body { self.compile_stmt(s)?; }
        if self.current_block_needs_terminator() {
            self.builder.build_unconditional_branch(merge_bb)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        }

        // elif chains
        let mut current_else_bb = first_else;
        for (i, arm) in elif_arms.iter().enumerate() {
            self.builder.position_at_end(current_else_bb);
            let elif_then = self.context.append_basic_block(parent, &format!("elif{}_then", i));
            let next = if i + 1 < elif_arms.len() {
                self.context.append_basic_block(parent, &format!("elif{}", i + 1))
            } else if else_body.is_some() {
                self.context.append_basic_block(parent, "else")
            } else {
                merge_bb
            };

            let (c, _) = self.compile_expr(&arm.condition)?;
            self.builder.build_conditional_branch(c.into_int_value(), elif_then, next)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

            self.builder.position_at_end(elif_then);
            for s in &arm.body { self.compile_stmt(s)?; }
            if self.current_block_needs_terminator() {
                self.builder.build_unconditional_branch(merge_bb)
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            }
            current_else_bb = next;
        }

        // else
        if let Some(else_stmts) = else_body {
            let else_bb = current_else_bb;
            self.builder.position_at_end(else_bb);
            for s in else_stmts { self.compile_stmt(s)?; }
            if self.current_block_needs_terminator() {
                self.builder.build_unconditional_branch(merge_bb)
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            }
        }

        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    fn compile_for_range(&mut self, var: &str, from: &Expr, to: &Expr, body: &[Stmt], label: &Option<String>)
        -> Result<(), CodegenError>
    {
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let (from_val, _) = self.compile_expr(from)?;
        let (to_val, _) = self.compile_expr(to)?;

        let i64_ty = self.context.i64_type();
        let ptr = self.builder.build_alloca(i64_ty, var)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        self.builder.build_store(ptr, from_val)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        self.variables.insert(var.to_string(), (ptr, CyType::Int));

        let header_bb = self.context.append_basic_block(parent, "for_header");
        let body_bb = self.context.append_basic_block(parent, "for_body");
        let exit_bb = self.context.append_basic_block(parent, "for_exit");

        self.builder.build_unconditional_branch(header_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        // header: check i < to
        self.builder.position_at_end(header_bb);
        let cur = self.builder.build_load(i64_ty, ptr, "i")
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        let cmp = self.builder.build_int_compare(IntPredicate::SLT, cur.into_int_value(), to_val.into_int_value(), "cmp")
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        self.builder.build_conditional_branch(cmp, body_bb, exit_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        // body
        self.builder.position_at_end(body_bb);
        self.loop_stack.push(LoopContext { exit_block: exit_bb, label: label.clone() });
        for s in body { self.compile_stmt(s)?; }
        self.loop_stack.pop();

        // increment
        if self.current_block_needs_terminator() {
            let cur = self.builder.build_load(i64_ty, ptr, "i_inc")
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            let one = i64_ty.const_int(1, false);
            let next = self.builder.build_int_add(cur.into_int_value(), one, "next")
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            self.builder.build_store(ptr, next)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            self.builder.build_unconditional_branch(header_bb)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        }

        self.builder.position_at_end(exit_bb);
        Ok(())
    }

    fn compile_for_c(&mut self, init: &Stmt, cond: &Expr, update: &Stmt, body: &[Stmt], label: &Option<String>)
        -> Result<(), CodegenError>
    {
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        self.compile_stmt(init)?;

        let header_bb = self.context.append_basic_block(parent, "forc_header");
        let body_bb = self.context.append_basic_block(parent, "forc_body");
        let exit_bb = self.context.append_basic_block(parent, "forc_exit");

        self.builder.build_unconditional_branch(header_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        self.builder.position_at_end(header_bb);
        let (cond_val, _) = self.compile_expr(cond)?;
        self.builder.build_conditional_branch(cond_val.into_int_value(), body_bb, exit_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        self.builder.position_at_end(body_bb);
        self.loop_stack.push(LoopContext { exit_block: exit_bb, label: label.clone() });
        for s in body { self.compile_stmt(s)?; }
        self.loop_stack.pop();

        if self.current_block_needs_terminator() {
            self.compile_stmt(update)?;
            self.builder.build_unconditional_branch(header_bb)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        }

        self.builder.position_at_end(exit_bb);
        Ok(())
    }

    fn compile_while(&mut self, condition: &Expr, body: &[Stmt], label: &Option<String>)
        -> Result<(), CodegenError>
    {
        let parent = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let header_bb = self.context.append_basic_block(parent, "while_header");
        let body_bb = self.context.append_basic_block(parent, "while_body");
        let exit_bb = self.context.append_basic_block(parent, "while_exit");

        self.builder.build_unconditional_branch(header_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        self.builder.position_at_end(header_bb);
        let (cond_val, _) = self.compile_expr(condition)?;
        self.builder.build_conditional_branch(cond_val.into_int_value(), body_bb, exit_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        self.builder.position_at_end(body_bb);
        self.loop_stack.push(LoopContext { exit_block: exit_bb, label: label.clone() });
        for s in body { self.compile_stmt(s)?; }
        self.loop_stack.pop();

        if self.current_block_needs_terminator() {
            self.builder.build_unconditional_branch(header_bb)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        }

        self.builder.position_at_end(exit_bb);
        Ok(())
    }

    fn compile_fun_decl(&mut self, name: &str, params: &[Param], _return_type: &Option<CyType>, body: &[Stmt])
        -> Result<(), CodegenError>
    {
        let func = self.module.get_function(name)
            .ok_or_else(|| CodegenError::UndefinedFunction(name.into()))?;
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // save outer scope
        let saved_vars = self.variables.clone();

        // alloca params so they're mutable
        for (i, param) in params.iter().enumerate() {
            let llvm_ty = self.cy_type_to_llvm(&param.type_ann);
            let ptr = self.builder.build_alloca(llvm_ty, &param.name)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            let arg_val = func.get_nth_param(i as u32).unwrap();
            self.builder.build_store(ptr, arg_val)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            self.variables.insert(param.name.clone(), (ptr, param.type_ann.clone()));
        }

        for s in body { self.compile_stmt(s)?; }

        // if no explicit return, add void return or default
        if self.current_block_needs_terminator() {
            if _return_type.is_some() {
                let zero = self.context.i64_type().const_int(0, false);
                self.builder.build_return(Some(&zero))
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            } else {
                self.builder.build_return(None)
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            }
        }

        self.variables = saved_vars;
        Ok(())
    }

    fn compile_return(&mut self, expr: &Option<Expr>) -> Result<(), CodegenError> {
        match expr {
            Some(e) => {
                let (val, _) = self.compile_expr(e)?;
                self.builder.build_return(Some(&val))
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            }
            None => {
                self.builder.build_return(None)
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn compile_break(&mut self, label: &Option<String>) -> Result<(), CodegenError> {
        let exit_bb = match label {
            // labeled break — walk the stack to find the named loop
            Some(lbl) => {
                self.loop_stack.iter().rev()
                    .find(|ctx| ctx.label.as_deref() == Some(lbl.as_str()))
                    .ok_or_else(|| CodegenError::Unsupported(
                        format!("no loop labeled '{}' in scope", lbl)
                    ))?
                    .exit_block
            }
            // unlabeled break — exit the innermost loop
            None => {
                self.loop_stack.last()
                    .ok_or_else(|| CodegenError::Unsupported("break outside loop".into()))?
                    .exit_block
            }
        };
        self.builder.build_unconditional_branch(exit_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        Ok(())
    }

    fn current_block_needs_terminator(&self) -> bool {
        self.builder.get_insert_block()
            .map_or(true, |bb| bb.get_terminator().is_none())
    }

    /// Emits the `end_when` guard after a block exits.
    ///
    /// Syntax: `endif when (cond): value`
    ///
    /// Semantics: if `cond` is true at the block's exit point,
    /// immediately return `value` from the enclosing function.
    /// Otherwise fall through to the next statement.
    fn compile_end_when(&mut self, end_when: &Option<EndWhen>) -> Result<(), CodegenError> {
        let ew = match end_when {
            Some(ew) => ew,
            None => return Ok(()), // nothing to do
        };

        let parent = self.builder
            .get_insert_block()
            .and_then(|b| b.get_parent())
            .ok_or_else(|| CodegenError::Unsupported("end_when outside function".into()))?;

        // evaluate the guard condition
        let (cond_val, _) = self.compile_expr(&ew.condition)?;

        // two destinations: early-return block vs fall-through block
        let early_bb    = self.context.append_basic_block(parent, "endwhen_early");
        let continue_bb = self.context.append_basic_block(parent, "endwhen_cont");

        self.builder
            .build_conditional_branch(cond_val.into_int_value(), early_bb, continue_bb)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        // early-return path: evaluate value and return it
        self.builder.position_at_end(early_bb);
        let (ret_val, _) = self.compile_expr(&ew.value)?;
        self.builder
            .build_return(Some(&ret_val))
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        // fall-through path: continue normal execution
        self.builder.position_at_end(continue_bb);
        Ok(())
    }
}
