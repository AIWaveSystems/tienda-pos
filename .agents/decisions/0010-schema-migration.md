# 0010 Migracion de esquema SQL

Al conectar se crean las tablas que falten y se agregan las columnas nuevas consultando `information_schema`. Nunca se borran ni cambian columnas.
