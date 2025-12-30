# Guía de Despliegue a Testnet

Esta guía explica cómo desplegar ambos contratos (Marketplace y ReportesView) a una testnet de Polkadot/Substrate.

## Prerequisitos

1. Rust y Cargo instalados
2. `cargo-contract` instalado: `cargo install cargo-contract --force`
3. Una cuenta en la testnet (Shibuya, Rococo, etc.) con fondos para gas

## Compilación

### 1. Compilar el contrato Marketplace

```bash
cargo contract build --manifest-path Cargo.toml
```

Esto generará `target/ink/marketplace.wasm` y `target/ink/marketplace.json`

### 2. Compilar el contrato ReportesView

El contrato ReportesView está en `reportes_view.rs`. Para compilarlo, necesitas crear un `Cargo.toml` separado o usar un workspace.

**Opción A: Compilar como binario separado**

Crea `Cargo.toml.reportes`:

```toml
[package]
name = "reportes-view"
version = "0.1.0"
authors = ["[your_name] <[your_email]>"]
edition = "2021"

[lib]
name = "reportes_view"
path = "reportes_view.rs"
crate-type = ["cdylib"]

[dependencies]
ink = { version = "5.1.1", default-features = false }

[features]
default = ["std"]
std = [
    "ink/std",
]
ink-as-dependency = []
```

Luego compila:

```bash
cargo contract build --manifest-path Cargo.toml.reportes
```

**Opción B: Usar un workspace (recomendado)**

Crea un `Cargo.toml` en la raíz:

```toml
[workspace]
members = ["marketplace", "reportes-view"]

[profile.release]
overflow-checks = true
lto = true
```

## Despliegue a Testnet

### Paso 1: Desplegar Marketplace

1. Sube el contrato a la testnet usando `cargo contract` o un UI como [Polkadot.js Apps](https://polkadot.js.org/apps)

```bash
cargo contract upload --suri //Alice --url wss://rpc.shibuya.astar.network
cargo contract instantiate --suri //Alice --url wss://rpc.shibuya.astar.network
```

2. **Guarda el AccountId del contrato Marketplace desplegado**. Lo necesitarás en el siguiente paso.

### Paso 2: Desplegar ReportesView

1. Usa el AccountId del Marketplace como parámetro del constructor:

```bash
# Primero, necesitas el AccountId del Marketplace (ejemplo: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY)
cargo contract instantiate \
  --suri //Alice \
  --url wss://rpc.shibuya.astar.network \
  --args 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

## Uso en Testnet

Una vez desplegados ambos contratos:

### Actualizar la dirección del Marketplace (si es necesario)

Si necesitas cambiar la dirección del Marketplace después del despliegue:

```rust
// Desde Polkadot.js o usando cargo-contract
reportes_view.actualizar_marketplace(nueva_direccion_marketplace)
```

### Ejemplos de llamadas

1. **Obtener top 5 vendedores:**
```rust
let top_vendedores = reportes_view.top_5_vendedores();
```

2. **Obtener productos más vendidos:**
```rust
let productos = reportes_view.productos_mas_vendidos();
```

3. **Obtener estadísticas por categoría:**
```rust
let stats = reportes_view.estadisticas_por_categoria();
```

## Testing Local

Para probar localmente antes de desplegar:

```bash
# Ejecutar tests del Marketplace
cargo test

# Los tests de ReportesView están incluidos en reportes_view.rs
# Ejecutan con valores por defecto cuando el Marketplace no está disponible
```

## Notas Importantes

1. **Asegúrate de que los tipos coincidan**: Los tipos `Producto` y `ReputacionData` en `reportes_view.rs` deben coincidir exactamente con los del contrato Marketplace.

2. **Manejo de errores**: El contrato ReportesView maneja errores de llamadas cross-contract retornando valores por defecto (0, vec vacío, None) para evitar fallos.

3. **Gas**: Las llamadas cross-contract consumen más gas. Asegúrate de tener suficientes fondos.

4. **Actualización del Marketplace**: Si actualizas el contrato Marketplace, puedes actualizar la dirección en ReportesView usando `actualizar_marketplace()`.

## Troubleshooting

### Error: "Contract not found"
- Verifica que el AccountId del Marketplace sea correcto
- Asegúrate de que el Marketplace esté desplegado en la misma red

### Error: "Call failed"
- Verifica que el contrato Marketplace tenga los métodos requeridos
- Revisa los selectores de los métodos (deben coincidir exactamente)

### Los resultados están vacíos
- Verifica que el Marketplace tenga datos
- Verifica que las llamadas cross-contract se estén ejecutando correctamente

