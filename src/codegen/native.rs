use crate::core::ast::{Program, Statement, Expression};
use crate::core::analyzer::{ZenithType};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{PointerValue, BasicValueEnum};
use std::collections::HashMap;

pub struct NativeCompiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> NativeCompiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        NativeCompiler {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    pub fn compile(&mut self, program: &Program) {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(basic_block);

        for stmt in &program.statements {
            self.compile_statement(stmt);
        }

        self.builder.build_return(Some(&i64_type.const_int(0, false)));
    }

    fn compile_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, value, .. } => {
                let val = self.compile_expression(value);
                let i64_type = self.context.i64_type();
                let ptr = self.builder.build_alloca(i64_type, name);
                self.builder.build_store(ptr, val);
                self.variables.insert(name.clone(), ptr);
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr);
            }
            _ => {}
        }
    }

    fn compile_expression(&self, expr: &Expression) -> BasicValueEnum<'ctx> {
        let i64_type = self.context.i64_type();
        match expr {
            Expression::IntegerLiteral(val) => {
                i64_type.const_int(*val as u64, false).into()
            }
            Expression::Variable(name) => {
                let ptr = self.variables.get(name).expect("Variable not found");
                self.builder.build_load(*ptr, name)
            }
            Expression::InfixExpression { left, operator, right } => {
                let l_val = self.compile_expression(left).into_int_value();
                let r_val = self.compile_expression(right).into_int_value();
                match operator.as_str() {
                    "+" => self.builder.build_int_add(l_val, r_val, "tmpadd").into(),
                    "-" => self.builder.build_int_sub(l_val, r_val, "tmpsub").into(),
                    "*" => self.builder.build_int_mul(l_val, r_val, "tmpmul").into(),
                    _ => i64_type.const_int(0, false).into(),
                }
            }
            _ => i64_type.const_int(0, false).into(),
        }
    }

    pub fn run_jit(&self) {
    }
}
