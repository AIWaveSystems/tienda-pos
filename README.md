# Ventas POS Rust

Sistema de punto de venta e inventario de escritorio, escrito en Rust y
multiplataforma (Windows, Linux y macOS). Es una reimplementacion de
[Ventas POS](https://github.com/AIWaveSystems/ventas-pos), escrito originalmente
en Python por [Ilesandres](https://github.com/Ilesandres).

## Que sera

- Las mismas vistas y reglas de negocio del original: ventas, inventario,
  categorias, clientes, historial, informes, usuarios, permisos e impresion.
- Soporte real de cada almacenamiento, aprovechando lo propio de cada motor en
  lugar de guardar JSON dentro de una columna de texto.
- Un unico binario nativo por plataforma, sin interprete ni runtime externo.

El port se construye primero hasta igualar al original y despues se mejora.

## Ademas del original

- Anulacion de ventas y movimientos de inventario con autor.
- Punto de venta con catalogo, cambio y notas.
- Informes por periodo con grafico, utilidad estimada y vendedores.
- Tema claro y oscuro, columnas ordenables y exportacion a CSV.

## Almacenamientos

| Motor | Modelo |
|---|---|
| JSON local | Un archivo por coleccion, escritura atomica |
| MySQL | Tablas relacionales, tipos nativos, indices y transacciones |
| PostgreSQL | Tablas relacionales, tipos nativos, indices y transacciones |
| MongoDB | Documentos nativos con `Decimal128` e indices |
| Google Drive | Archivos por coleccion con deteccion de conflictos |
| Dropbox | Archivos por coleccion con control de revision |

## Arquitectura

Workspace de Cargo en capas. Las dependencias apuntan siempre hacia el dominio.

| Crate | Responsabilidad | Depende de |
|---|---|---|
| `pos-domain` | Entidades, permisos y reglas puras | nada interno |
| `pos-config` | Configuracion, rutas y credenciales cifradas | domain |
| `pos-storage` | Contratos de persistencia y un adaptador por motor | domain, config |
| `pos-services` | Casos de uso, informes e impresion | domain, config, storage |
| `pos-desktop` | Interfaz grafica por componentes y vistas | todos |

## Estructura

```
ventas-pos-rust/
  .agents/          decisiones, cambios, entidades y convenciones
  assets/           iconos
  crates/
    domain/
    config/
    storage/
    services/
    desktop/
  Cargo.toml
```

## Uso

```bash
cargo run --release -p pos-desktop
```

Usuario inicial `admin`, contrasena `admin123`; el sistema obliga a cambiarla.

| Sistema | Configuracion | Datos |
|---|---|---|
| Windows | `%APPDATA%\VentasPOSRust` | `%LOCALAPPDATA%\VentasPOSRust` |
| Linux / macOS | `~/.config/ventas-pos-rust` | `~/.local/share/ventas-pos-rust` |
| Portable (`portable.txt`) | `config/` | `data/` |

En Linux hace falta `libdbus-1-dev` para compilar. Google Drive y Dropbox
necesitan un archivo `.env` basado en `.env.example`.

## Pruebas

```bash
cargo test --workspace
docker compose up --detach --wait
cargo test -p pos-storage --test databases -- --include-ignored
```

Para usar un servidor propio se definen `POS_TEST_POSTGRES_HOST`, `_PORT`,
`_USER`, `_PASSWORD` y `_DATABASE`. Las pruebas borran datos: la base debe
terminar en `_test`.

El estado real de cada comprobacion esta en `.agents/parity.md`.

## Ramas

| Rama | Uso |
|---|---|
| `main` | Versiones estables |
| `develop` | Integracion |
| `feat/*`, `fix/*`, `docs/*` | Trabajo; se integran en `develop` |

## Creditos

- **Agente principal del port: [Claude](https://claude.com/claude-code)**
  (Anthropic). Analisis del original, arquitectura, codigo, pruebas y
  documentacion de esta version en Rust.
- **Autor del proyecto original y responsable de este:**
  [Ilesandres](https://github.com/Ilesandres).

Las decisiones del agente quedan registradas en la carpeta `.agents`.

## Licencia

MIT. Ver [LICENSE](LICENSE).
