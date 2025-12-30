#!/bin/bash

# Script de ejemplo para desplegar los contratos a testnet
# Ajusta las variables según tu entorno

# Configuración
TESTNET_URL="wss://rpc.shibuya.astar.network"
DEPLOYER_SURI="//Alice"  # Cambia esto por tu seed phrase o account
MARKETPLACE_CODE_HASH=""  # Se llenará después de subir el contrato

echo "=== Paso 1: Subiendo contrato Marketplace ==="
cargo contract upload \
  --suri "$DEPLOYER_SURI" \
  --url "$TESTNET_URL" \
  --execute

# Guarda el code_hash del output y actualiza MARKETPLACE_CODE_HASH

echo "=== Paso 2: Instanciando contrato Marketplace ==="
MARKETPLACE_ADDRESS=$(cargo contract instantiate \
  --suri "$DEPLOYER_SURI" \
  --url "$TESTNET_URL" \
  --constructor new \
  --execute \
  --output-json | jq -r '.contract')

echo "Marketplace desplegado en: $MARKETPLACE_ADDRESS"

echo "=== Paso 3: Subiendo contrato ReportesView ==="
# Nota: Necesitas compilar reportes_view.rs primero
# cargo contract build --manifest-path Cargo.toml.reportes

cargo contract upload \
  --suri "$DEPLOYER_SURI" \
  --url "$TESTNET_URL" \
  --execute

echo "=== Paso 4: Instanciando contrato ReportesView ==="
REPORTES_ADDRESS=$(cargo contract instantiate \
  --suri "$DEPLOYER_SURI" \
  --url "$TESTNET_URL" \
  --constructor new \
  --args "$MARKETPLACE_ADDRESS" \
  --execute \
  --output-json | jq -r '.contract')

echo "ReportesView desplegado en: $REPORTES_ADDRESS"
echo ""
echo "=== Direcciones de los contratos ==="
echo "Marketplace: $MARKETPLACE_ADDRESS"
echo "ReportesView: $REPORTES_ADDRESS"
echo ""
echo "Guarda estas direcciones para uso posterior"

