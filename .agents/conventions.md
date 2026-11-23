# Convenciones

- Codigo e identificadores en ingles; textos de interfaz en espanol.
- Sin comentarios en el codigo: los nombres explican y las decisiones van aqui.
- `cargo fmt`, `cargo clippy -- -D warnings` y `cargo test` antes de integrar.
- Commits minimos con el formato de `.gitmessage`, un cambio por commit.
- Las ramas salen de `develop` y vuelven con merge `--no-ff`.
- Los errores usan tipos propios; nada de `unwrap` en codigo de produccion.
- El proyecto original es de solo lectura.
