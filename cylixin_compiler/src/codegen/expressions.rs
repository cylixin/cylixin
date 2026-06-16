use inkwell::values::BasicValueEnum;
use crate::ast::*;
use super::compiler::{Compiler, CodegenError};

impl<'ctx> Compiler<'ctx> {
    pub fn compile_expr(&self, expr: &Expr) -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError> {
        match expr {
            Expr::IntLit(n) => {
                let val = self.context.i64_type().const_int(*n as u64, true);
                Ok((val.into(), CyType::Int))
            }
            Expr::LongLit(n) => {
                let val = self.context.i64_type().const_int(*n as u64, true);
                Ok((val.into(), CyType::Long))
            }
            Expr::FloatLit(f) => {
                let val = self.context.f64_type().const_float(*f);
                Ok((val.into(), CyType::Float))
            }
            Expr::BoolLit(b) => {
                let val = self.context.bool_type().const_int(*b as u64, false);
                Ok((val.into(), CyType::Bool))
            }
            Expr::CharLit(c) => {
                let val = self.context.i8_type().const_int(*c as u64, false);
                Ok((val.into(), CyType::Char))
            }
            Expr::StringLit(s) => {
                let val = self.builder.build_global_string_ptr(s, "str")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((val.as_pointer_value().into(), CyType::StringType))
            }
            Expr::NullLit => {
                let val = self.context.i64_type().const_int(0, false);
                Ok((val.into(), CyType::Null))
            }
            Expr::Ident(name) => {
                let (ptr, ty) = self.variables.get(name)
                    .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))?;
                let llvm_ty = self.cy_type_to_llvm(ty);
                let val = self.builder.build_load(llvm_ty, *ptr, name)
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((val, ty.clone()))
            }
            Expr::BinaryOp { left, op, right } => self.compile_binary(left, op, right),
            Expr::UnaryOp { op, expr } => self.compile_unary(op, expr),
            Expr::Call { name, args } => self.compile_call(name, args),
            Expr::Grouped(inner) => self.compile_expr(inner),
            Expr::ArrayLit(_) | Expr::Index { .. } => {
                Err(CodegenError::Unsupported("arrays/indexing not yet implemented".into()))
            }
        }
    }

    fn compile_binary(&self, left: &Expr, op: &BinaryOp, right: &Expr)
        -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError>
    {
        let (lv, lt) = self.compile_expr(left)?;
        let (rv, _rt) = self.compile_expr(right)?;

        // For strict equality, if types differ, it's always false.
        // If they match, we evaluate it exactly like normal equality.
        let effective_op = if op == &BinaryOp::StrictEq {
            if lt != _rt {
                return Ok((self.context.bool_type().const_int(0, false).into(), CyType::Bool));
            }
            &BinaryOp::Eq
        } else {
            op
        };

        match (&lt, effective_op) {
            // int/long arithmetic
            (CyType::Int | CyType::Long, BinaryOp::Add) => {
                let r = self.builder.build_int_add(lv.into_int_value(), rv.into_int_value(), "add")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            (CyType::Int | CyType::Long, BinaryOp::Sub) => {
                let r = self.builder.build_int_sub(lv.into_int_value(), rv.into_int_value(), "sub")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            (CyType::Int | CyType::Long, BinaryOp::Mul) => {
                let r = self.builder.build_int_mul(lv.into_int_value(), rv.into_int_value(), "mul")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            (CyType::Int | CyType::Long, BinaryOp::Div) => {
                let r = self.builder.build_int_signed_div(lv.into_int_value(), rv.into_int_value(), "div")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            (CyType::Int | CyType::Long, BinaryOp::Mod) => {
                let r = self.builder.build_int_signed_rem(lv.into_int_value(), rv.into_int_value(), "mod")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            // int comparisons
            (CyType::Int | CyType::Long, BinaryOp::Eq) => {
                let r = self.builder.build_int_compare(inkwell::IntPredicate::EQ, lv.into_int_value(), rv.into_int_value(), "eq")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (CyType::Int | CyType::Long, BinaryOp::NotEq) => {
                let r = self.builder.build_int_compare(inkwell::IntPredicate::NE, lv.into_int_value(), rv.into_int_value(), "ne")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (CyType::Int | CyType::Long, BinaryOp::Lt) => {
                let r = self.builder.build_int_compare(inkwell::IntPredicate::SLT, lv.into_int_value(), rv.into_int_value(), "lt")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (CyType::Int | CyType::Long, BinaryOp::Gt) => {
                let r = self.builder.build_int_compare(inkwell::IntPredicate::SGT, lv.into_int_value(), rv.into_int_value(), "gt")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (CyType::Int | CyType::Long, BinaryOp::LtEq) => {
                let r = self.builder.build_int_compare(inkwell::IntPredicate::SLE, lv.into_int_value(), rv.into_int_value(), "le")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (CyType::Int | CyType::Long, BinaryOp::GtEq) => {
                let r = self.builder.build_int_compare(inkwell::IntPredicate::SGE, lv.into_int_value(), rv.into_int_value(), "ge")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            // float arithmetic
            (CyType::Float, BinaryOp::Add) => {
                let r = self.builder.build_float_add(lv.into_float_value(), rv.into_float_value(), "fadd")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Float))
            }
            (CyType::Float, BinaryOp::Sub) => {
                let r = self.builder.build_float_sub(lv.into_float_value(), rv.into_float_value(), "fsub")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Float))
            }
            (CyType::Float, BinaryOp::Mul) => {
                let r = self.builder.build_float_mul(lv.into_float_value(), rv.into_float_value(), "fmul")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Float))
            }
            (CyType::Float, BinaryOp::Div) => {
                let r = self.builder.build_float_div(lv.into_float_value(), rv.into_float_value(), "fdiv")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Float))
            }
            // float comparisons
            (CyType::Float, BinaryOp::Lt) => {
                let r = self.builder.build_float_compare(inkwell::FloatPredicate::OLT, lv.into_float_value(), rv.into_float_value(), "flt")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (CyType::Float, BinaryOp::Gt) => {
                let r = self.builder.build_float_compare(inkwell::FloatPredicate::OGT, lv.into_float_value(), rv.into_float_value(), "fgt")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (CyType::Float, BinaryOp::Eq) => {
                let r = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lv.into_float_value(), rv.into_float_value(), "feq")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            // logical
            (_, BinaryOp::And) => {
                let r = self.builder.build_and(lv.into_int_value(), rv.into_int_value(), "and")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            (_, BinaryOp::Or) => {
                let r = self.builder.build_or(lv.into_int_value(), rv.into_int_value(), "or")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            // exponentiation — call C's pow(f64, f64) -> f64
            (CyType::Int | CyType::Long, BinaryOp::Pow) => {
                let pow_fn = self.module.get_function("pow")
                    .ok_or_else(|| CodegenError::UndefinedFunction("pow".into()))?;
                let f64_type = self.context.f64_type();
                let lf = self.builder.build_signed_int_to_float(lv.into_int_value(), f64_type, "lf")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                let rf = self.builder.build_signed_int_to_float(rv.into_int_value(), f64_type, "rf")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                let result = self.builder.build_call(pow_fn, &[lf.into(), rf.into()], "pow")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?
                    .try_as_basic_value();
                match result {
                    inkwell::values::ValueKind::Basic(v) => {
                        let i = self.builder.build_float_to_signed_int(v.into_float_value(), self.context.i64_type(), "ipow")
                            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                        Ok((i.into(), lt))
                    }
                    _ => Err(CodegenError::LLVMError("pow returned no value".into())),
                }
            }
            (CyType::Float, BinaryOp::Pow) => {
                let pow_fn = self.module.get_function("pow")
                    .ok_or_else(|| CodegenError::UndefinedFunction("pow".into()))?;
                let result = self.builder.build_call(pow_fn, &[lv.into(), rv.into()], "fpow")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?
                    .try_as_basic_value();
                match result {
                    inkwell::values::ValueKind::Basic(v) => Ok((v, CyType::Float)),
                    _ => Err(CodegenError::LLVMError("pow returned no value".into())),
                }
            }
            _ => Err(CodegenError::Unsupported(format!("binary op {:?} on {:?}", op, lt))),
        }
    }

    fn compile_unary(&self, op: &UnaryOp, expr: &Expr)
        -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError>
    {
        let (val, ty) = self.compile_expr(expr)?;
        match (op, &ty) {
            (UnaryOp::Neg, CyType::Int | CyType::Long) => {
                let r = self.builder.build_int_neg(val.into_int_value(), "neg")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), ty))
            }
            (UnaryOp::Neg, CyType::Float) => {
                let r = self.builder.build_float_neg(val.into_float_value(), "fneg")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), ty))
            }
            (UnaryOp::Not, CyType::Bool) => {
                let r = self.builder.build_not(val.into_int_value(), "not")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), CyType::Bool))
            }
            _ => Err(CodegenError::Unsupported(format!("unary {:?} on {:?}", op, ty))),
        }
    }

    pub fn compile_call(&self, name: &str, args: &[Expr])
        -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError>
    {
        match name {
            "write" | "writeln" => {
                let newline = name == "writeln";
                for arg in args {
                    let (val, ty) = self.compile_expr(arg)?;
                    let fmt = match (&ty, newline && args.len() == 1) {
                        (CyType::Int | CyType::Long, true)  => "%lld\n",
                        (CyType::Int | CyType::Long, false) => "%lld",
                        (CyType::Float, true)  => "%f\n",
                        (CyType::Float, false) => "%f",
                        (CyType::StringType, true)  => "%s\n",
                        (CyType::StringType, false) => "%s",
                        (CyType::Bool, true)  => "%lld\n",
                        (CyType::Bool, false) => "%lld",
                        (CyType::Char, true)  => "%c\n",
                        (CyType::Char, false) => "%c",
                        _ => "%lld\n",
                    };
                    let fmt_ptr = self.builder.build_global_string_ptr(fmt, "fmt")
                        .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                    let printf = self.module.get_function("printf")
                        .ok_or_else(|| CodegenError::UndefinedFunction("printf".into()))?;
                    // widen i1/i8 to i64 for printf
                    let print_val = match &ty {
                        CyType::Bool => {
                            let widened = self.builder.build_int_z_extend(val.into_int_value(), self.context.i64_type(), "zbool")
                                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                            widened.into()
                        }
                        CyType::Char => {
                            let widened = self.builder.build_int_z_extend(val.into_int_value(), self.context.i64_type(), "zchar")
                                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                            widened.into()
                        }
                        _ => val,
                    };
                    self.builder.build_call(
                        printf,
                        &[fmt_ptr.as_pointer_value().into(), print_val.into()],
                        "printf_call",
                    ).map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                }
                // write/writeln returns void, but we need a value — return 0
                let zero = self.context.i64_type().const_int(0, false);
                Ok((zero.into(), CyType::Int))
            }
            _ => {
                let (func, ret_ty) = self.functions.get(name)
                    .ok_or_else(|| CodegenError::UndefinedFunction(name.into()))?;
                let func = *func;
                let ret_ty = ret_ty.clone().unwrap_or(CyType::Int);
                let mut compiled_args = Vec::new();
                for arg in args {
                    let (val, _) = self.compile_expr(arg)?;
                    compiled_args.push(val.into());
                }
                let call = self.builder.build_call(func, &compiled_args, "call")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                match call.try_as_basic_value() {
                    inkwell::values::ValueKind::Basic(v) => Ok((v, ret_ty)),
                    inkwell::values::ValueKind::Instruction(_) => {
                        let zero = self.context.i64_type().const_int(0, false);
                        Ok((zero.into(), CyType::Int))
                    }
                }
            }
        }
    }
}
