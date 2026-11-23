# Arquitectura

| Capa | Puede depender de | No conoce |
|---|---|---|
| domain | nada interno | disco, red, interfaz |
| config | domain | persistencia de negocio, interfaz |
| storage | domain, config | interfaz |
| services | domain, config, storage | interfaz, motor concreto |
| desktop | todas | SQL, HTTP, formato de archivos |

## Flujo

1. `desktop` carga la configuracion y pide a `storage::factory` el motor activo.
2. Los servicios reciben `Arc<dyn Store>` y no saben que motor hay detras.
3. Las vistas lanzan las cargas en un hilo y consultan el resultado en cada
   cuadro, para que la red no congele la interfaz.
4. Los motores no locales se envuelven en `MirroredStore`, que mantiene una copia
   JSON local de respaldo.
