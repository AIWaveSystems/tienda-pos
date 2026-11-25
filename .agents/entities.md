# Entidades

Identificador: 12 caracteres hexadecimales. Fechas: hora local sin zona con
microsegundos. Dinero: decimal exacto, serializado como numero en JSON.

| Entidad | Campos |
|---|---|
| Category | id, name, description, active, created_at, updated_at |
| Client | id, name, email, phone, address, tax_id, notes, active, created_at, updated_at |
| Product | id, name, description, price, cost, stock, min_stock, category_id, barcode, active, created_at, updated_at |
| Sale | id, client_id, client_name, items, subtotal, tax, discount, total, payment_method, notes, seller, voided_at, created_at |
| StockMovement | id, product_id, product_name, kind, quantity, reference, author, created_at |
| SaleItem | product_id, product_name, quantity, unit_price, subtotal |
| User | id, username, password_hash, full_name, role, permissions, is_default, active, last_login, created_at, updated_at |

## Reglas

- Stock bajo: `stock <= min_stock`.
- Margen: `(price - cost) / cost * 100`; cero si `cost <= 0`.
- `tax = subtotal * tasa`, `total = subtotal + tax - discount`.
- `SaleItem` copia nombre y precio: borrar un producto no altera el historial.
- Hash de contrasena: `pbkdf2$rondas$salt$digest`, SHA-256, 120000 rondas.
- Debe existir siempre al menos un administrador activo.

- `StockMovement.quantity` lleva signo; la suma explica cada cambio de stock.
- Una venta se anula una sola vez y deja de contar en informes.

## Roles

| Rol | Permisos |
|---|---|
| admin | todos |
| vendedor | dashboard, clientes, ventas, historial |
| bodega | dashboard, inventario, informes |
| personalizado | seleccion manual |
