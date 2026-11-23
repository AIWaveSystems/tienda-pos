# Almacenamiento

| Motor | Modelo | Concurrencia |
|---|---|---|
| local_json | archivo con `schema_version` e `items` | bloqueo en proceso y reemplazo atomico |
| mysql | tablas `pos_*`, `DECIMAL(18,4)`, `DATETIME(6)` | transacciones InnoDB |
| postgresql | tablas `pos_*`, `NUMERIC(18,4)`, `TIMESTAMP` | transacciones |
| mongodb | colecciones `pos_*`, `Decimal128` | atomicidad por documento |
| google_drive | archivos en carpeta `VentasPOSRust` | comparacion de `md5Checksum` |
| dropbox | archivos en la carpeta de la app | `rev` verificado por el servidor |

`pos_sale_items` y `pos_user_permissions` son tablas hijas con clave foranea y
borrado en cascada. Las referencias historicas (cliente, producto, categoria) no
llevan clave foranea porque deben sobrevivir al borrado.
