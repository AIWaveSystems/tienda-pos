# Paridad con el original

Referencia: rama `feat/externals-integrations`, commit `2bde9b7`.

| Area | Estado |
|---|---|
| Entidades, permisos, hash de contrasena | Igual; lee los JSON del original |
| Configuracion, modo portable, credenciales cifradas | Igual |
| Autenticacion y usuarios | Igual |
| Categorias, clientes, inventario, ventas, informes | Igual |
| Comprobantes A4 y termicos, listados e informes | Igual |
| Vistas, atajos, paginacion, select con busqueda, paleta | Igual en funcion |
| Respaldo local, migracion entre motores, OAuth | Igual |

## Diferencias intencionales

- Interfaz en una sola ventana: login y carga no son ventanas aparte.
- Permitir stock negativo funciona al confirmar; el original lo bloqueaba.
- El total de impresion usa todas las coincidencias, como dice el README original;
  su codigo imprimia solo la pagina visible.
- Guardar configuracion con la contrasena vacia conserva la ya guardada.
- El acceso a la nube se renueva al expirar; el original fallaba tras una hora.
- Un archivo remoto ilegible produce error; el original lo trataba como vacio.

## Verificacion

| Comprobado | Como |
|---|---|
| JSON local, respaldo y migracion | contrato comun de `Store` |
| Google Drive y Dropbox | servidor falso, nunca contra cuentas reales |
| Interfaz | `egui_kittest` y capturas renderizadas |

| Sin comprobar | Motivo |
|---|---|
| MySQL, PostgreSQL y MongoDB reales | Docker sin virtualizacion en el equipo |
| Ejecutable en Windows | Smart App Control bloqueo el binario sin firma |
| Linux y macOS | solo definido en CI, aun sin ejecutar |
| Impresion fisica y llavero del sistema | requieren sesion de usuario |
