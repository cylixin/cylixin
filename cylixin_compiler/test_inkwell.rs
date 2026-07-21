use inkwell::context::Context;
fn main() {
    let ctx = Context::create();
    let _ = ctx.ptr_type(inkwell::AddressSpace::default());
}
