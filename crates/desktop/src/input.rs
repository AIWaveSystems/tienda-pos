use eframe::egui::{Context, Key, Modifiers};

pub fn command(ctx: &Context, key: Key) -> bool {
    ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, key))
}

pub fn pressed(ctx: &Context, key: Key) -> bool {
    ctx.input_mut(|input| input.consume_key(Modifiers::NONE, key))
}

pub fn free(ctx: &Context) -> bool {
    ctx.memory(|memory| memory.focused().is_none())
}

pub fn modal_open(ctx: &Context) -> bool {
    ctx.memory(|memory| memory.top_modal_layer().is_some())
}
