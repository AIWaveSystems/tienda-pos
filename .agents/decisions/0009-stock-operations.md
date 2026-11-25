# 0009 Operaciones de stock

Venta, anulacion y ajuste son una unica operacion de `Store` (`StockOperation`) que mueve stock y registra los movimientos juntos. La anulacion se protege con `voided_at IS NULL`, de modo que dos cajas no pueden anular la misma venta. Reemplaza el metodo de 0007.
