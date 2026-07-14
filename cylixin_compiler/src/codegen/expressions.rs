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
            Expr::ArrayLit(elements) => {
                self.compile_array_lit(elements)
            }
            Expr::SetLit(elements) => {
                self.compile_set_lit(elements)
            }
            Expr::DictLit(pairs) => {
                self.compile_dict_lit(pairs)
            }
            Expr::Index { collection, index } => {
                self.compile_index(collection, index)
            }
        }
    }

    fn compile_array_lit(&self, elements: &[Expr])
        -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError>
    {
        let i64_type = self.context.i64_type();

        let malloc_fn = self.module.get_function("malloc")
            .ok_or_else(|| CodegenError::UndefinedFunction("malloc".into()))?;

        let count = elements.len() as u64;
        // Layout: [length: i64][elem0: i64][elem1: i64]...
        // Total bytes = (count + 1) * 8
        let total_slots = i64_type.const_int(count + 1, false);
        let eight = i64_type.const_int(8, false);
        let alloc_size = self.builder.build_int_mul(total_slots, eight, "alloc_size")
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        let raw_ptr = self.builder.build_call(malloc_fn, &[alloc_size.into()], "arr_alloc")
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?
            .try_as_basic_value();
        let raw_ptr = match raw_ptr {
            inkwell::values::ValueKind::Basic(v) => v.into_pointer_value(),
            _ => return Err(CodegenError::LLVMError("malloc returned no value".into())),
        };

        // Cast to i64* for convenient GEP
        let i64_ptr = self.builder.build_pointer_cast(raw_ptr, i64_type.ptr_type(inkwell::AddressSpace::default()), "arr_i64ptr")
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        // Store length at slot 0
        let len_val = i64_type.const_int(count, false);
        self.builder.build_store(i64_ptr, len_val)
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

        // Compile and store each element at slots 1..count
        let mut elem_ty = CyType::Int; // default, will be overridden by first element
        for (i, elem_expr) in elements.iter().enumerate() {
            let (val, ty) = self.compile_expr(elem_expr)?;
            if i == 0 { elem_ty = ty.clone(); }

            let slot_index = i64_type.const_int((i as u64) + 1, false);
            let elem_ptr = unsafe {
                self.builder.build_gep(i64_type, i64_ptr, &[slot_index], &format!("elem_ptr_{}", i))
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?
            };

            // Store: cast the value to i64 if needed
            let store_val = self.value_to_i64_slot(&val, &ty)?;
            self.builder.build_store(elem_ptr, store_val)
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        }

        // Return the raw pointer (i8*) and the array type with element info
        Ok((raw_ptr.into(), CyType::Arr(Some(Box::new(elem_ty)))))
    }

    fn compile_index(&self, collection: &Expr, index: &Expr)
        -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError>
    {
        let i64_type = self.context.i64_type();
        let (coll_val, coll_ty) = self.compile_expr(collection)?;

        match &coll_ty {
            CyType::Arr(inner) => {
                let elem_ty = inner.as_ref().map(|t| *t.clone()).unwrap_or(CyType::Int);
                let (idx_val, _) = self.compile_expr(index)?;

                let i64_ptr = self.builder.build_pointer_cast(
                    coll_val.into_pointer_value(),
                    i64_type.ptr_type(inkwell::AddressSpace::default()),
                    "arr_i64ptr"
                ).map_err(|e| CodegenError::LLVMError(e.to_string()))?;

                let one = i64_type.const_int(1, false);
                let slot = self.builder.build_int_add(idx_val.into_int_value(), one, "slot")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

                let elem_ptr = unsafe {
                    self.builder.build_gep(i64_type, i64_ptr, &[slot], "idx_ptr")
                        .map_err(|e| CodegenError::LLVMError(e.to_string()))?
                };

                let raw_val = self.builder.build_load(i64_type, elem_ptr, "idx_val")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

                let typed_val = self.i64_slot_to_value(raw_val, &elem_ty)?;
                Ok((typed_val, elem_ty))
            }
            CyType::Dic(_, val_ty) => {
                let val_elem_ty = val_ty.as_ref().map(|t| *t.clone()).unwrap_or(CyType::Int);
                let dict_get = self.module.get_function("cy_dict_get")
                    .ok_or_else(|| CodegenError::UndefinedFunction("cy_dict_get".into()))?;

                let (key_val, key_ty) = self.compile_expr(index)?;
                let key_i64 = self.value_to_i64_slot(&key_val, &key_ty)?;

                let result = self.builder.build_call(dict_get, &[coll_val.into(), key_i64.into()], "dict_val")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                let raw = match result.try_as_basic_value() {
                    inkwell::values::ValueKind::Basic(v) => v,
                    _ => return Err(CodegenError::LLVMError("cy_dict_get returned no value".into())),
                };

                let typed_val = self.i64_slot_to_value(raw, &val_elem_ty)?;
                Ok((typed_val, val_elem_ty))
            }
            _ => Err(CodegenError::Unsupported(format!("indexing on {:?}", coll_ty))),
        }
    }

    fn compile_set_lit(&self, elements: &[Expr])
        -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError>
    {
        let set_new = self.module.get_function("cy_set_new")
            .ok_or_else(|| CodegenError::UndefinedFunction("cy_set_new".into()))?;
        let set_add = self.module.get_function("cy_set_add")
            .ok_or_else(|| CodegenError::UndefinedFunction("cy_set_add".into()))?;

        let result = self.builder.build_call(set_new, &[], "set_ptr")
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        let set_ptr = match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(v) => v,
            _ => return Err(CodegenError::LLVMError("cy_set_new returned no value".into())),
        };

        let mut elem_ty = CyType::Int;
        for (i, elem_expr) in elements.iter().enumerate() {
            let (val, ty) = self.compile_expr(elem_expr)?;
            if i == 0 { elem_ty = ty.clone(); }
            let val_i64 = self.value_to_i64_slot(&val, &ty)?;
            self.builder.build_call(set_add, &[set_ptr.into(), val_i64.into()], "set_add")
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        }

        Ok((set_ptr, CyType::Set(Some(Box::new(elem_ty)))))
    }

    fn compile_dict_lit(&self, pairs: &[(Expr, Expr)])
        -> Result<(BasicValueEnum<'ctx>, CyType), CodegenError>
    {
        let dict_new = self.module.get_function("cy_dict_new")
            .ok_or_else(|| CodegenError::UndefinedFunction("cy_dict_new".into()))?;
        let dict_set = self.module.get_function("cy_dict_set")
            .ok_or_else(|| CodegenError::UndefinedFunction("cy_dict_set".into()))?;

        let result = self.builder.build_call(dict_new, &[], "dict_ptr")
            .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        let dict_ptr = match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(v) => v,
            _ => return Err(CodegenError::LLVMError("cy_dict_new returned no value".into())),
        };

        let mut key_ty = CyType::Int;
        let mut val_ty = CyType::Int;
        for (i, (key_expr, val_expr)) in pairs.iter().enumerate() {
            let (k, kt) = self.compile_expr(key_expr)?;
            let (v, vt) = self.compile_expr(val_expr)?;
            if i == 0 { key_ty = kt.clone(); val_ty = vt.clone(); }
            let k_i64 = self.value_to_i64_slot(&k, &kt)?;
            let v_i64 = self.value_to_i64_slot(&v, &vt)?;
            self.builder.build_call(dict_set, &[dict_ptr.into(), k_i64.into(), v_i64.into()], "dict_set")
                .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
        }

        Ok((dict_ptr, CyType::Dic(Some(Box::new(key_ty)), Some(Box::new(val_ty)))))
    }


    /// Converts any value to an i64 for uniform storage in array slots.
    fn value_to_i64_slot(&self, val: &BasicValueEnum<'ctx>, ty: &CyType)
        -> Result<inkwell::values::IntValue<'ctx>, CodegenError>
    {
        let i64_type = self.context.i64_type();
        match ty {
            CyType::Int | CyType::Long => Ok(val.into_int_value()),
            CyType::Float => {
                self.builder.build_bitcast(*val, i64_type, "f2i")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))
                    .map(|v| v.into_int_value())
            }
            CyType::Bool => {
                self.builder.build_int_z_extend(val.into_int_value(), i64_type, "b2i")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))
            }
            CyType::Char => {
                self.builder.build_int_z_extend(val.into_int_value(), i64_type, "c2i")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))
            }
            CyType::StringType | CyType::Arr(_) | CyType::Set(_) | CyType::Dic(_, _) => {
                self.builder.build_ptr_to_int(val.into_pointer_value(), i64_type, "p2i")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))
            }
            _ => Ok(val.into_int_value()),
        }
    }

    /// Converts an i64 slot value back to the expected element type.
    fn i64_slot_to_value(&self, raw: BasicValueEnum<'ctx>, ty: &CyType)
        -> Result<BasicValueEnum<'ctx>, CodegenError>
    {
        let i64_type = self.context.i64_type();
        match ty {
            CyType::Int | CyType::Long => Ok(raw),
            CyType::Float => {
                self.builder.build_bitcast(raw, self.context.f64_type(), "i2f")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))
            }
            CyType::Bool => {
                let trunc = self.builder.build_int_truncate(raw.into_int_value(), self.context.bool_type(), "i2b")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(trunc.into())
            }
            CyType::Char => {
                let trunc = self.builder.build_int_truncate(raw.into_int_value(), self.context.i8_type(), "i2c")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(trunc.into())
            }
            CyType::StringType | CyType::Arr(_) | CyType::Set(_) | CyType::Dic(_, _) => {
                let ptr_type = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
                let ptr = self.builder.build_int_to_ptr(raw.into_int_value(), ptr_type, "i2p")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok(ptr.into())
            }
            _ => Ok(raw),
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
            // string concatenation
            (CyType::StringType, BinaryOp::Add) => {
                let strlen_fn = self.module.get_function("strlen")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strlen".into()))?;
                let malloc_fn = self.module.get_function("malloc")
                    .ok_or_else(|| CodegenError::UndefinedFunction("malloc".into()))?;
                let strcpy_fn = self.module.get_function("strcpy")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strcpy".into()))?;
                let strcat_fn = self.module.get_function("strcat")
                    .ok_or_else(|| CodegenError::UndefinedFunction("strcat".into()))?;

                let extract_val = |result: inkwell::values::CallSiteValue<'ctx>| -> Result<inkwell::values::BasicValueEnum<'ctx>, CodegenError> {
                    match result.try_as_basic_value() {
                        inkwell::values::ValueKind::Basic(v) => Ok(v),
                        _ => Err(CodegenError::LLVMError("function returned no value".into())),
                    }
                };

                let len_l = extract_val(self.builder.build_call(strlen_fn, &[lv.into()], "len_l")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?)?;
                let len_r = extract_val(self.builder.build_call(strlen_fn, &[rv.into()], "len_r")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?)?;

                let total_len = self.builder.build_int_add(len_l.into_int_value(), len_r.into_int_value(), "total_len")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                let one = self.context.i64_type().const_int(1, false);
                let malloc_size = self.builder.build_int_add(total_len, one, "malloc_size")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

                let new_str = extract_val(self.builder.build_call(malloc_fn, &[malloc_size.into()], "new_str")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?)?;

                self.builder.build_call(strcpy_fn, &[new_str.into(), lv.into()], "strcpy_call")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                self.builder.build_call(strcat_fn, &[new_str.into(), rv.into()], "strcat_call")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;

                Ok((new_str, CyType::StringType))
            }
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
            // bitwise operators
            (CyType::Int | CyType::Long, BinaryOp::BitAnd) => {
                let r = self.builder.build_and(lv.into_int_value(), rv.into_int_value(), "bitand")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            (CyType::Int | CyType::Long, BinaryOp::BitOr) => {
                let r = self.builder.build_or(lv.into_int_value(), rv.into_int_value(), "bitor")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            (CyType::Int | CyType::Long, BinaryOp::Shl) => {
                let r = self.builder.build_left_shift(lv.into_int_value(), rv.into_int_value(), "shl")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
            }
            (CyType::Int | CyType::Long, BinaryOp::Shr) => {
                let r = self.builder.build_right_shift(lv.into_int_value(), rv.into_int_value(), true, "shr")
                    .map_err(|e| CodegenError::LLVMError(e.to_string()))?;
                Ok((r.into(), lt))
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
