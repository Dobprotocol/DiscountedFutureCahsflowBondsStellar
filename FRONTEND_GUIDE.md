# DOB Liquidity - Frontend Guide

## 🎉 Frontend Completo Implementado

Se ha creado una aplicación web completa con React + TypeScript para interactuar con todos los contratos del proyecto DOB Liquidity.

## 📦 Estructura del Proyecto

```
frontend/
├── src/
│   ├── components/          # Componentes de UI
│   │   ├── Header.tsx       # Header con conexión de wallet
│   │   ├── OracleInfo.tsx   # Información del Oracle (NAV, Risk)
│   │   ├── SwapInterface.tsx # Interfaz de compra/venta
│   │   ├── LiquidityManager.tsx # Gestión de liquidez
│   │   ├── LiquidNodes.tsx  # Dashboard de Liquid Nodes
│   │   └── UserBalances.tsx # Balances del usuario
│   ├── hooks/               # React Hooks personalizados
│   │   ├── useWallet.ts     # Gestión de Freighter wallet
│   │   └── useContracts.ts  # Gestión de estado de contratos
│   ├── utils/               # Utilidades
│   │   ├── stellar.ts       # Funciones de Stellar SDK
│   │   └── contracts.ts     # Service para interactuar con contratos
│   ├── types/               # Tipos TypeScript
│   │   └── index.ts
│   ├── App.tsx              # Componente principal
│   ├── main.tsx             # Punto de entrada
│   └── index.css            # Estilos globales
├── package.json
├── vite.config.ts
├── tailwind.config.js
└── README.md
```

## ✨ Características Implementadas

### 1. 🔐 Conexión de Wallet (Freighter)
- Detección automática de Freighter
- Conexión/desconexión de wallet
- Detección de red (testnet/mainnet)
- Muestra dirección del usuario

### 2. 📊 Oracle Dashboard
- **NAV (Net Asset Value)**: Precio justo del token
- **Risk (Default Risk)**: Porcentaje de riesgo
- Actualización en tiempo real cada 30 segundos
- Indicador visual de nivel de riesgo (bajo/medio/alto/muy alto)

### 3. 💱 Interfaz de Swap (Compra/Venta)

#### Compra (AfterSwap Hook)
- Ingresa cantidad de USDC
- Calcula DOB a recibir basado en NAV
- Muestra fee (1% DEX)
- Explica que los tokens se mintean al NAV
- Los 99% van al operador de infraestructura

#### Venta (BeforeSwap Hook)
- Ingresa cantidad de DOB
- Calcula USDC a recibir
- Muestra si usará pool o Liquid Nodes
- Fee dinámico basado en riesgo del Oracle
- Los tokens se queman después del swap

### 4. 💧 Gestión de Liquidez

#### Agregar Liquidez
- Ingresa USDC y DOB
- Recibe LP shares
- Gana fees proporcionales de todos los swaps

#### Remover Liquidez
- Quema LP shares
- Recibe USDC y DOB proporcionalmente

### 5. 🖥️ Monitor de Liquid Nodes
- Lista todos los Liquid Nodes registrados
- Muestra balance de USDC disponible
- Muestra holdings de DOB
- Estado online/offline
- Última cotización (si disponible)

### 6. 💰 Panel de Balances
- Balance de USDC
- Balance de DOB
- LP Shares
- Actualización automática

## 🚀 Inicio Rápido

### Opción 1: Setup Automático

```bash
# Desde la raíz del proyecto
./scripts/setup-frontend.sh

# Luego
cd frontend
npm run dev
```

### Opción 2: Setup Manual

```bash
cd frontend

# Instalar dependencias
npm install

# Copiar archivo de ejemplo
cp .env.example .env

# Editar .env con las direcciones de tus contratos
nano .env

# Iniciar servidor de desarrollo
npm run dev
```

## 📝 Configuración de Contratos

### Desde deployed-contracts.env

Después de ejecutar `./scripts/deploy-and-test.sh`, copia las direcciones:

```bash
source deployed-contracts.env

# Luego usa estas variables en frontend/.env:
VITE_TOKEN_ID=$TOKEN_ID
VITE_ORACLE_ID=$ORACLE_ID
VITE_POOL_ID=$POOL_ID
VITE_USDC_ID=$USDC_ID
VITE_LN1_ID=$LN1_ID
VITE_LN2_ID=$LN2_ID
VITE_NETWORK=$NETWORK
```

### Configuración desde la UI

Si no tienes las variables de entorno:
1. Abre la aplicación
2. Se mostrará modal de configuración
3. Pega las direcciones de los contratos
4. Haz clic en "Save Configuration"

## 🎨 Tecnologías Utilizadas

- **React 18**: Framework de UI
- **TypeScript**: Tipado estático
- **Vite**: Build tool rápido
- **Tailwind CSS**: Estilos utility-first
- **Stellar SDK**: Interacción con blockchain
- **Freighter**: Wallet de Stellar
- **Lucide React**: Iconos

## 📱 Funcionalidades de la UI

### Diseño Responsive
- Desktop: Grid de 3 columnas
- Tablet: Grid de 2 columnas
- Mobile: Stack vertical

### Animaciones
- Loading states
- Hover effects
- Transitions suaves
- Spin en botón de refresh

### Feedback Visual
- Success/error messages
- Loading spinners
- Disabled states
- Tooltips informativos

## 🔄 Flujo de Usuario Típico

### 1. Primera Vez
```
1. Conectar wallet Freighter
2. Configurar direcciones de contratos
3. Ver dashboard con información del Oracle
```

### 2. Comprar DOB
```
1. Ir a "Swap Tokens"
2. Seleccionar modo "Buy"
3. Ingresar cantidad de USDC
4. Ver estimación de DOB
5. Hacer clic en "Buy DOB"
6. Aprobar en Freighter
7. Recibir DOB minteados al NAV
```

### 3. Vender DOB (con Liquid Nodes)
```
1. Ir a "Swap Tokens"
2. Seleccionar modo "Sell"
3. Ingresar cantidad de DOB
4. Sistema verifica liquidez del pool
5. Si insuficiente, consulta Liquid Nodes
6. Muestra mejor cotización combinada
7. Hacer clic en "Sell DOB"
8. Aprobar en Freighter
9. Recibir USDC, tokens DOB se queman
```

### 4. Proveer Liquidez
```
1. Ir a "Manage Liquidity"
2. Seleccionar modo "Add"
3. Ingresar USDC y DOB
4. Hacer clic en "Add Liquidity"
5. Recibir LP shares
6. Comenzar a ganar fees de trading
```

## 🛡️ Seguridad

- No almacena claves privadas
- Todas las transacciones requieren aprobación en Freighter
- Validación de inputs
- Manejo de errores robusto
- Simulación de transacciones antes de enviar

## 🐛 Troubleshooting

### Freighter no detectado
```
1. Instalar extensión de Freighter
2. Refrescar página
3. Hacer clic en "Connect Freighter"
```

### Transacciones fallan
```
1. Verificar que estás en la red correcta (testnet)
2. Confirmar que tienes balance suficiente
3. Verificar direcciones de contratos
4. Revisar consola del navegador para detalles
```

### Datos no cargan
```
1. Hacer clic en botón "Refresh"
2. Verificar conexión a internet
3. Verificar que contratos están desplegados
4. Revisar consola para errores
```

## 📊 Métricas y Analytics

El frontend muestra en tiempo real:
- NAV actual del token
- Nivel de riesgo
- Liquidez disponible en el pool
- Balances de Liquid Nodes
- Tus balances personales
- LP shares

## 🔮 Próximas Mejoras Potenciales

- [ ] Gráficos de histórico de precios
- [ ] Histórico de transacciones
- [ ] Calculadora de APY para LP providers
- [ ] Notificaciones de transacciones
- [ ] Modo oscuro/claro
- [ ] Multi-idioma (EN/ES)
- [ ] Export de datos a CSV
- [ ] Integración con otros wallets (Albedo, etc.)

## 📞 Soporte

Para problemas o preguntas:
1. Revisa la consola del navegador
2. Verifica las direcciones de contratos
3. Confirma que Freighter está en testnet
4. Revisa logs del servidor de desarrollo

## 🎓 Recursos Adicionales

- [Stellar SDK Docs](https://stellar.github.io/js-stellar-sdk/)
- [Freighter Wallet](https://www.freighter.app/)
- [Soroban Docs](https://soroban.stellar.org/docs)
- [React Docs](https://react.dev/)
- [Tailwind CSS](https://tailwindcss.com/)

---

**¡El frontend está 100% funcional y listo para usar!** 🚀
