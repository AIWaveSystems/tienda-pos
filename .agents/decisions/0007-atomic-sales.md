# 0007 Venta atomica

El original guarda la venta y despues resta stock producto a producto. Aqui es una sola operacion de `Store`. SQL: transaccion con `UPDATE ... AND stock >= n`, en orden de producto e inserciones puras para evitar interbloqueos. MongoDB: decremento condicional y compensacion, porque las transacciones exigen replica set. Archivos: bloqueo en proceso y restauracion si falla el segundo archivo; entre equipos protege la revision del proveedor.
