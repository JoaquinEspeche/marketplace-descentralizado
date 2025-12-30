#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
/// Módulo que implementa un marketplace descentralizado usando ink!
mod marketplace {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;

    /// Enum que representa los roles posibles de un usuario dentro del marketplace.
    #[derive(Clone, PartialEq, Eq, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Roles { 
        Comprador, 
        Vendedor, 
        Ambos 
    }

    impl Roles {
        /// Verifica si el rol incluye permisos de comprador.
        pub fn es_comprador(&self) -> bool {
            matches!(self, Roles::Comprador | Roles::Ambos)
        }

        /// Verifica si el rol incluye permisos de vendedor.
        pub fn es_vendedor(&self) -> bool {
            matches!(self, Roles::Vendedor | Roles::Ambos)
        }

        /// Agrega un nuevo rol al rol actual.
        /// Retorna el nuevo rol combinado o un error si se intenta quitar un rol.
        /// Solo permite agregar roles, no quitarlos (ej: Ambos -> Comprador no está permitido).
        pub fn agregar_rol(&self, nuevo_rol: Roles) -> Result<Roles, ContractError> {
            match (self, nuevo_rol) {
                // Si ya es Ambos, siempre queda Ambos (no se puede quitar)
                (Roles::Ambos, _) => Ok(Roles::Ambos),
                // Si se intenta agregar Ambos, resulta en Ambos
                (_, Roles::Ambos) => Ok(Roles::Ambos),
                // Agregar Vendedor a Comprador o viceversa resulta en Ambos
                (Roles::Comprador, Roles::Vendedor) => Ok(Roles::Ambos),
                (Roles::Vendedor, Roles::Comprador) => Ok(Roles::Ambos),
                // Si el rol es el mismo, se mantiene
                (r1, r2) if r1 == &r2 => Ok(self.clone()),
                // Cualquier otro caso (no debería ocurrir con los roles actuales)
                _ => Ok(self.clone()),
            }
        }
    }

    /// Enum que representa los estados posibles de una orden de compra.
    #[derive(Clone, PartialEq, Eq , Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum EstadoOrden { Pendiente, Enviado, Recibido, Cancelada }

    /// Enum que representa errores posibles en las operaciones del contrato.
    #[derive(Clone, PartialEq, Eq, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ContractError {
        YaRegistrado,
        UsuarioNoRegistrado,
        NoVendedor,
        NoAutorizado,
        ProductoNoEncontrado,
        StockInsuficiente,
        OrdenNoExiste,
        EstadoInvalido,
        Overflow,
        DatosInvalidos,
        NoSePuedeQuitarRol,
        CalificacionInvalida,
        YaCalificado,
        OrdenNoRecibida,
    }

    /// Estructura que almacena las calificaciones de una orden.
    #[derive(Clone, PartialEq, Eq, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct CalificacionesOrden {
        /// Calificación del comprador al vendedor (1-5).
        pub calificacion_comprador: Option<u8>,
        /// Calificación del vendedor al comprador (1-5).
        pub calificacion_vendedor: Option<u8>,
    }

    /// Estructura que representa la reputación acumulada de un usuario.
    #[derive(Clone, PartialEq, Eq, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct ReputacionData {
        /// Total de calificaciones recibidas como comprador.
        pub total_calificaciones_comprador: u32,
        /// Suma de todas las calificaciones recibidas como comprador.
        pub suma_calificaciones_comprador: u128,
        /// Total de calificaciones recibidas como vendedor.
        pub total_calificaciones_vendedor: u32,
        /// Suma de todas las calificaciones recibidas como vendedor.
        pub suma_calificaciones_vendedor: u128,
    }

    impl ReputacionData {
        /// Crea una nueva instancia de ReputacionData vacía.
        pub fn new() -> Self {
            Self {
                total_calificaciones_comprador: 0,
                suma_calificaciones_comprador: 0,
                total_calificaciones_vendedor: 0,
                suma_calificaciones_vendedor: 0,
            }
        }

        /// Calcula el promedio de reputación como comprador.
        pub fn promedio_comprador(&self) -> Option<u128> {
            if self.total_calificaciones_comprador > 0 {
                self.suma_calificaciones_comprador
                    .checked_div(self.total_calificaciones_comprador as u128)
            } else {
                None
            }
        }

        /// Calcula el promedio de reputación como vendedor.
        pub fn promedio_vendedor(&self) -> Option<u128> {
            if self.total_calificaciones_vendedor > 0 {
                self.suma_calificaciones_vendedor
                    .checked_div(self.total_calificaciones_vendedor as u128)
            } else {
                None
            }
        }

        /// Agrega una calificación como comprador.
        pub fn agregar_calificacion_comprador(&mut self, calificacion: u8) -> Result<(), ContractError> {
            self.total_calificaciones_comprador = self.total_calificaciones_comprador
                .checked_add(1)
                .ok_or(ContractError::Overflow)?;
            self.suma_calificaciones_comprador = self.suma_calificaciones_comprador
                .checked_add(calificacion as u128)
                .ok_or(ContractError::Overflow)?;
            Ok(())
        }

        /// Agrega una calificación como vendedor.
        pub fn agregar_calificacion_vendedor(&mut self, calificacion: u8) -> Result<(), ContractError> {
            self.total_calificaciones_vendedor = self.total_calificaciones_vendedor
                .checked_add(1)
                .ok_or(ContractError::Overflow)?;
            self.suma_calificaciones_vendedor = self.suma_calificaciones_vendedor
                .checked_add(calificacion as u128)
                .ok_or(ContractError::Overflow)?;
            Ok(())
        }
    }

    impl Default for ReputacionData {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Estructura que representa un producto publicado por un vendedor.
    #[derive(Clone, PartialEq, Eq, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Producto {
        pub nombre: String,
        pub descripcion: String,
        pub precio: u128,
        pub cantidad: u32,
        pub categoria: String,
        pub vendedor: AccountId,
    }

    impl Producto {
        /// Valida que los datos del producto sean correctos.
        /// Retorna un error si algún campo está vacío o tiene valor 0.
        pub fn validar(&self) -> Result<(), ContractError> {
            if self.nombre.is_empty() 
                || self.descripcion.is_empty() 
                || self.categoria.is_empty() {
                return Err(ContractError::DatosInvalidos);
            }
            if self.precio == 0 || self.cantidad == 0 {
                return Err(ContractError::DatosInvalidos);
            }
            Ok(())
        }

        /// Aumenta el stock del producto en la cantidad especificada.
        pub fn aumentar_stock(&mut self, cantidad: u32) -> Result<(), ContractError> {
            self.cantidad = self.cantidad
                .checked_add(cantidad)
                .ok_or(ContractError::Overflow)?;
            Ok(())
        }
    }

    /// Estructura que representa una orden de compra realizada por un comprador.
    #[derive(Clone, PartialEq, Eq, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Orden {
        pub comprador: AccountId,
        pub vendedor: AccountId,
        pub producto_id: u128,
        pub cantidad: u32,
        pub estado: EstadoOrden,
        pub comprador_acepta_cancelar: bool,
        pub vendedor_acepta_cancelar: bool,
    }

    impl Orden {
        /// Verifica si la orden puede ser cancelada.
        pub fn puede_cancelarse(&self) -> bool {
            matches!(self.estado, EstadoOrden::Pendiente | EstadoOrden::Enviado)
        }

        /// Marca la orden como cancelada si ambos participantes están de acuerdo.
        pub fn marcar_cancelada_si_ambos_aceptan(&mut self) -> bool {
            if self.comprador_acepta_cancelar && self.vendedor_acepta_cancelar {
                self.estado = EstadoOrden::Cancelada;
                true
            } else {
                false
            }
        }
    }

    /// Contrato Marketplace donde los usuarios pueden registrarse, publicar productos y crear órdenes.
    #[ink(storage)]
    pub struct Marketplace {
        /// Mapea una cuenta a su rol (Comprador, Vendedor o Ambos).
        roles: Mapping<AccountId, Roles>,

        /// Mapea un ID de producto a su estructura de datos.
        productos: Mapping<u128, Producto>,

        /// Mapea un usuario con los IDs de productos que publicó.
        productos_por_usuario: Mapping<AccountId, Vec<u128>>,

        /// ID del próximo producto a registrar.
        siguiente_producto_id: u128,

        /// Mapea un ID de orden a su estructura.
        ordenes: Mapping<u128, Orden>,

        /// Mapea un usuario con las órdenes que creó.
        ordenes_por_usuario: Mapping<AccountId, Vec<u128>>,

        /// ID de la próxima orden a registrar.
        siguiente_orden_id: u128,

        /// Mapea un ID de orden a sus calificaciones.
        calificaciones_por_orden: Mapping<u128, CalificacionesOrden>,

        /// Mapea un usuario a su reputación acumulada.
        reputaciones: Mapping<AccountId, ReputacionData>,

        /// Mapea un producto a la cantidad de veces que ha sido vendido.
        ventas_por_producto: Mapping<u128, u32>,

        /// Mapea una categoría a estadísticas de ventas y calificaciones.
        /// La clave es la categoría como String.
        /// El valor es (total_ventas, suma_calificaciones, cantidad_calificaciones).
        estadisticas_por_categoria: Mapping<String, (u32, u128, u32)>,

        /// Lista de todos los usuarios registrados (para reportes).
        /// Usamos un Mapping como lista indexada: (índice) -> AccountId.
        usuarios_registrados: Mapping<u32, AccountId>,
        /// Contador de usuarios registrados (para saber cuántos hay).
        contador_usuarios: u32,
    }

    impl Marketplace {
        /// Crea una nueva instancia del contrato con estructuras vacías.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                roles: Mapping::default(),
                productos: Mapping::default(),
                productos_por_usuario: Mapping::default(),
                siguiente_producto_id: 1,
                ordenes: Mapping::default(),
                ordenes_por_usuario: Mapping::default(),
                siguiente_orden_id: 1,
                calificaciones_por_orden: Mapping::default(),
                reputaciones: Mapping::default(),
                ventas_por_producto: Mapping::default(),
                estadisticas_por_categoria: Mapping::default(),
                usuarios_registrados: Mapping::default(),
                contador_usuarios: 0,
            }
        }

        /// Registra un nuevo usuario con un rol determinado.
        #[ink(message)]
        pub fn registrar_usuario(&mut self, rol: Roles) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._registrar_usuario(caller, rol)
        }

        /// Modifica el rol de un usuario ya registrado.
        #[ink(message)]
        pub fn modificar_rol(&mut self, nuevo_rol: Roles) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._modificar_rol(caller, nuevo_rol)
        }

        /// Devuelve el rol de un usuario específico.
        #[ink(message)]
        pub fn obtener_rol(&self, usuario: AccountId) -> Option<Roles> {
            self._obtener_rol(usuario)
        }

        /// Devuelve el estado de una orden específica.
        #[ink(message)]
        pub fn obtener_estado_orden(&self, orden_id: u128) -> Option<EstadoOrden> {
            self._obtener_estado_orden(orden_id)
        }

        /// Publica un nuevo producto para el usuario que llama.
        #[ink(message)]
        pub fn publicar_producto(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: u128,
            cantidad: u32,
            categoria: String,
        ) -> Result<u128, ContractError> {
            let caller = self.env().caller();
            self._publicar_producto(caller, nombre, descripcion, precio, cantidad, categoria)
        }

        /// Devuelve los productos publicados por el usuario que llama.
        #[ink(message)]
        pub fn ver_mis_productos(&self) -> Vec<(u128, Producto)> {
            let caller = self.env().caller();
            self._ver_mis_productos(caller)
        }

        /// Devuelve todos los productos publicados en el marketplace.
        #[ink(message)]
        pub fn ver_todos_los_productos(&self) -> Vec<(u128, Producto)> {
            self._ver_todos_los_productos()
        }

        /// Crea una nueva orden de compra para el producto indicado.
        #[ink(message)]
        pub fn crear_orden_de_compra(&mut self, producto_id: u128, cantidad: u32) -> Result<u128, ContractError> {
            let caller = self.env().caller();
            self._crear_orden(caller, producto_id, cantidad)
        }

        /// Marca una orden como enviada (solo el vendedor puede hacerlo).
        #[ink(message)]
        pub fn marcar_orden_enviada(&mut self, orden_id: u128) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._marcar_enviada(caller, orden_id)
        }

        /// Marca una orden como recibida (solo el comprador puede hacerlo).
        #[ink(message)]
        pub fn marcar_orden_recibida(&mut self, orden_id: u128) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._marcar_recibida(caller, orden_id)
        }

        /// El comprador solicita la cancelación de una orden.
        #[ink(message)]
        pub fn comprador_solicita_cancelacion(&mut self, orden_id: u128) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._solicitar_cancel_comprador(caller, orden_id)
        }

        /// El vendedor acepta la cancelación de una orden.
        #[ink(message)]
        pub fn vendedor_acepta_cancelacion(&mut self, orden_id: u128) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._aceptar_cancel_vendedor(caller, orden_id)
        }

        /// El comprador califica al vendedor después de recibir la orden.
        /// Solo se puede calificar si la orden está en estado Recibido.
        #[ink(message)]
        pub fn comprador_califica_vendedor(
            &mut self,
            orden_id: u128,
            calificacion: u8,
        ) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._calificar_vendedor(caller, orden_id, calificacion)
        }

        /// El vendedor califica al comprador después de recibir la orden.
        /// Solo se puede calificar si la orden está en estado Recibido.
        #[ink(message)]
        pub fn vendedor_califica_comprador(
            &mut self,
            orden_id: u128,
            calificacion: u8,
        ) -> Result<(), ContractError> {
            let caller = self.env().caller();
            self._calificar_comprador(caller, orden_id, calificacion)
        }

        /// Obtiene la reputación de un usuario.
        #[ink(message)]
        pub fn obtener_reputacion(&self, usuario: AccountId) -> Option<ReputacionData> {
            self._obtener_reputacion(usuario)
        }

        /// Obtiene la reputación promedio como comprador de un usuario.
        #[ink(message)]
        pub fn reputacion_como_comprador(&self, usuario: AccountId) -> Option<u128> {
            self._obtener_reputacion(usuario)
                .and_then(|r| r.promedio_comprador())
        }

        /// Obtiene la reputación promedio como vendedor de un usuario.
        #[ink(message)]
        pub fn reputacion_como_vendedor(&self, usuario: AccountId) -> Option<u128> {
            self._obtener_reputacion(usuario)
                .and_then(|r| r.promedio_vendedor())
        }

        /// Obtiene las calificaciones de una orden.
        #[ink(message)]
        pub fn obtener_calificaciones_orden(&self, orden_id: u128) -> Option<CalificacionesOrden> {
            self.calificaciones_por_orden.get(orden_id)
        }

        /// Obtiene la cantidad de ventas de un producto.
        #[ink(message)]
        pub fn obtener_ventas_producto(&self, producto_id: u128) -> u32 {
            self.ventas_por_producto.get(producto_id).unwrap_or(0)
        }

        /// Obtiene las estadísticas de una categoría.
        #[ink(message)]
        pub fn obtener_estadisticas_categoria(&self, categoria: String) -> Option<(u32, u128, u32)> {
            self.estadisticas_por_categoria.get(&categoria)
        }

        /// Obtiene la cantidad de órdenes de un usuario.
        #[ink(message)]
        pub fn cantidad_ordenes_usuario(&self, usuario: AccountId) -> u32 {
            self.ordenes_por_usuario
                .get(&usuario)
                .map(|v| v.len() as u32)
                .unwrap_or(0)
        }

        /// Obtiene todos los usuarios con reputación (para reportes).
        /// Retorna un vector de tuplas (usuario, reputacion_data).
        #[ink(message)]
        pub fn obtener_usuarios_con_reputacion(&self) -> Vec<(AccountId, ReputacionData)> {
            let mut resultado = Vec::new();
            for i in 0..self.contador_usuarios {
                if let Some(usuario) = self.usuarios_registrados.get(i) {
                    if let Some(reputacion) = self.reputaciones.get(usuario) {
                        resultado.push((usuario, reputacion));
                    }
                }
            }
            resultado
        }


        // ===== Funciones privadas =====

        /// Registra un nuevo usuario con el rol especificado.
        fn _registrar_usuario(
            &mut self, 
            caller: AccountId, 
            rol: Roles
        ) -> Result<(), ContractError> {
            if self.roles.contains(caller) {
                return Err(ContractError::YaRegistrado);
            }
            self.roles.insert(caller, &rol);
            // Agregar a la lista de usuarios registrados
            let index = self.contador_usuarios;
            self.usuarios_registrados.insert(index, &caller);
            self.contador_usuarios = index.checked_add(1).ok_or(ContractError::Overflow)?;
            Ok(())
        }

        /// Modifica el rol de un usuario, permitiendo solo agregar roles, no quitarlos.
        fn _modificar_rol(
            &mut self, 
            caller: AccountId, 
            nuevo_rol: Roles
        ) -> Result<(), ContractError> {
            let rol_actual = self.roles.get(caller)
                .ok_or(ContractError::UsuarioNoRegistrado)?;
            
            let rol_actualizado = rol_actual.agregar_rol(nuevo_rol)?;
            self.roles.insert(caller, &rol_actualizado);
            Ok(())
        }

        /// Obtiene el rol de un usuario.
        fn _obtener_rol(&self, usuario: AccountId) -> Option<Roles> {
            self.roles.get(usuario)
        }

        /// Obtiene el estado de una orden.
        fn _obtener_estado_orden(&self, orden_id: u128) -> Option<EstadoOrden> {
            self.ordenes.get(orden_id).map(|orden| orden.estado.clone())
        }
        /// Publica un nuevo producto validando que todos los campos sean válidos.
        fn _publicar_producto(
            &mut self,
            caller: AccountId,
            nombre: String,
            descripcion: String,
            precio: u128,
            cantidad: u32,
            categoria: String,
        ) -> Result<u128, ContractError> {
            let rol = self.roles.get(&caller);
            if !rol.map_or(false, |r| r.es_vendedor()) {
                return Err(ContractError::NoVendedor);
            }

            let producto = Producto {
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
                vendedor: caller,
            };

            // Validar que los datos del producto sean correctos
            producto.validar()?;

            let pid = self.siguiente_producto_id;
            self.productos.insert(pid, &producto);
            
            let mut lista = self.productos_por_usuario
                .get(&caller)
                .unwrap_or_default();
            lista.push(pid);
            self.productos_por_usuario.insert(&caller, &lista);
            
            self.siguiente_producto_id = pid
                .checked_add(1)
                .ok_or(ContractError::Overflow)?;
            Ok(pid)
        }

        /// Obtiene todos los productos publicados por un usuario.
        fn _ver_mis_productos(&self, caller: AccountId) -> Vec<(u128, Producto)> {
            self.productos_por_usuario
                .get(&caller)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|id| {
                    self.productos.get(id).map(|p| (id, p))
                })
                .collect()
        }

        /// Obtiene todos los productos publicados en el marketplace.
        fn _ver_todos_los_productos(&self) -> Vec<(u128, Producto)> {
            let mut acc = Vec::new();
            for id in 1..self.siguiente_producto_id {
                if let Some(p) = self.productos.get(id) {
                    acc.push((id, p));
                }
            }
            acc
        }

        /// Crea una nueva orden de compra validando stock y permisos.
        fn _crear_orden(
            &mut self, 
            comprador: AccountId, 
            producto_id: u128, 
            cantidad: u32
        ) -> Result<u128, ContractError> {
            // Validar que el usuario tenga permisos de comprador
            let rol = self.roles.get(&comprador);
            if !rol.map_or(false, |r| r.es_comprador()) {
                return Err(ContractError::NoAutorizado);
            }

            // Validar que la cantidad sea mayor que 0
            if cantidad == 0 {
                return Err(ContractError::StockInsuficiente);
            }

            // Obtener y validar el producto
            let mut producto = self.productos
                .get(producto_id)
                .ok_or(ContractError::ProductoNoEncontrado)?;
            
            if producto.cantidad < cantidad {
                return Err(ContractError::StockInsuficiente);
            }

            // Reducir el stock del producto
            producto.cantidad = producto.cantidad
                .checked_sub(cantidad)
                .ok_or(ContractError::Overflow)?;
            self.productos.insert(producto_id, &producto);

            // Crear la orden
            let oid = self.siguiente_orden_id;
            let orden = Orden {
                comprador,
                vendedor: producto.vendedor,
                producto_id,
                cantidad,
                estado: EstadoOrden::Pendiente,
                comprador_acepta_cancelar: false,
                vendedor_acepta_cancelar: false,
            };
            self.ordenes.insert(oid, &orden);

            // Agregar la orden a la lista del comprador
            let mut ordenes_usuario = self.ordenes_por_usuario
                .get(&comprador)
                .unwrap_or_default();
            ordenes_usuario.push(oid);
            self.ordenes_por_usuario.insert(&comprador, &ordenes_usuario);

            self.siguiente_orden_id = oid
                .checked_add(1)
                .ok_or(ContractError::Overflow)?;
            Ok(oid)
        }

        /// Marca una orden como enviada (solo puede ser Pendiente).
        fn _marcar_enviada(
            &mut self, 
            caller: AccountId, 
            orden_id: u128
        ) -> Result<(), ContractError> {
            let mut orden = self.ordenes
                .get(orden_id)
                .ok_or(ContractError::OrdenNoExiste)?;
            
            if orden.vendedor != caller {
                return Err(ContractError::NoAutorizado);
            }

            // No se puede pasar directamente a Recibido
            // Solo se puede enviar desde Pendiente
            if orden.estado != EstadoOrden::Pendiente {
                return Err(ContractError::EstadoInvalido);
            }

            orden.estado = EstadoOrden::Enviado;
            self.ordenes.insert(orden_id, &orden);
            Ok(())
        }

        /// Marca una orden como recibida (solo puede ser desde Enviado, no retrocede).
        fn _marcar_recibida(
            &mut self, 
            caller: AccountId, 
            orden_id: u128
        ) -> Result<(), ContractError> {
            let mut orden = self.ordenes
                .get(orden_id)
                .ok_or(ContractError::OrdenNoExiste)?;
            
            if orden.comprador != caller {
                return Err(ContractError::NoAutorizado);
            }

            // Solo se puede recibir desde Enviado
            // No se puede pasar directamente de Pendiente a Recibido
            if orden.estado != EstadoOrden::Enviado {
                return Err(ContractError::EstadoInvalido);
            }

            // Una vez recibido, no se puede retroceder
            orden.estado = EstadoOrden::Recibido;
            self.ordenes.insert(orden_id, &orden);

            // Registrar venta del producto
            let ventas_actuales = self.ventas_por_producto.get(orden.producto_id).unwrap_or(0);
            self.ventas_por_producto.insert(
                orden.producto_id,
                &ventas_actuales.checked_add(1).ok_or(ContractError::Overflow)?,
            );

            // Inicializar calificaciones vacías para esta orden
            let calificaciones = CalificacionesOrden {
                calificacion_comprador: None,
                calificacion_vendedor: None,
            };
            self.calificaciones_por_orden.insert(orden_id, &calificaciones);

            Ok(())
        }

        /// El comprador solicita la cancelación de una orden.
        /// Si ambos aceptan, se cancela y se devuelve el stock.
        fn _solicitar_cancel_comprador(
            &mut self, 
            caller: AccountId, 
            orden_id: u128
        ) -> Result<(), ContractError> {
            let mut orden = self.ordenes
                .get(orden_id)
                .ok_or(ContractError::OrdenNoExiste)?;
            
            if orden.comprador != caller {
                return Err(ContractError::NoAutorizado);
            }

            // Solo se puede cancelar si está Pendiente o Enviado
            if !orden.puede_cancelarse() {
                return Err(ContractError::EstadoInvalido);
            }

            orden.comprador_acepta_cancelar = true;
            
            // Si ambos aceptan, cancelar y devolver stock
            if orden.marcar_cancelada_si_ambos_aceptan() {
                self._devolver_stock(orden.producto_id, orden.cantidad)?;
            }
            
            self.ordenes.insert(orden_id, &orden);
            Ok(())
        }

        /// El vendedor acepta la cancelación de una orden.
        /// Si ambos aceptan, se cancela y se devuelve el stock.
        fn _aceptar_cancel_vendedor(
            &mut self, 
            caller: AccountId, 
            orden_id: u128
        ) -> Result<(), ContractError> {
            let mut orden = self.ordenes
                .get(orden_id)
                .ok_or(ContractError::OrdenNoExiste)?;
            
            if orden.vendedor != caller {
                return Err(ContractError::NoAutorizado);
            }

            // Solo se puede cancelar si está Pendiente o Enviado
            if !orden.puede_cancelarse() {
                return Err(ContractError::EstadoInvalido);
            }

            orden.vendedor_acepta_cancelar = true;
            
            // Si ambos aceptan, cancelar y devolver stock
            if orden.marcar_cancelada_si_ambos_aceptan() {
                self._devolver_stock(orden.producto_id, orden.cantidad)?;
            }
            
            self.ordenes.insert(orden_id, &orden);
            Ok(())
        }

        /// Devuelve stock a un producto cuando se cancela una orden.
        fn _devolver_stock(
            &mut self, 
            producto_id: u128, 
            cantidad: u32
        ) -> Result<(), ContractError> {
            let mut producto = self.productos
                .get(producto_id)
                .ok_or(ContractError::ProductoNoEncontrado)?;
            
            producto.aumentar_stock(cantidad)?;
            self.productos.insert(producto_id, &producto);
            Ok(())
        }

        /// Valida que una calificación esté en el rango válido (1-5).
        fn _validar_calificacion(calificacion: u8) -> Result<(), ContractError> {
            if calificacion < 1 || calificacion > 5 {
                return Err(ContractError::CalificacionInvalida);
            }
            Ok(())
        }

        /// El comprador califica al vendedor.
        fn _calificar_vendedor(
            &mut self,
            comprador: AccountId,
            orden_id: u128,
            calificacion: u8,
        ) -> Result<(), ContractError> {
            // Validar rango de calificación
            Self::_validar_calificacion(calificacion)?;

            // Obtener y validar la orden
            let orden = self.ordenes
                .get(orden_id)
                .ok_or(ContractError::OrdenNoExiste)?;

            // Verificar que el caller es el comprador
            if orden.comprador != comprador {
                return Err(ContractError::NoAutorizado);
            }

            // Solo se puede calificar si la orden está recibida
            if orden.estado != EstadoOrden::Recibido {
                return Err(ContractError::OrdenNoRecibida);
            }

            // Obtener calificaciones existentes
            let mut calificaciones = self.calificaciones_por_orden
                .get(orden_id)
                .ok_or(ContractError::EstadoInvalido)?;

            // Verificar que no haya calificado antes
            if calificaciones.calificacion_comprador.is_some() {
                return Err(ContractError::YaCalificado);
            }

            // Guardar la calificación
            calificaciones.calificacion_comprador = Some(calificacion);
            self.calificaciones_por_orden.insert(orden_id, &calificaciones);

            // Actualizar reputación del vendedor
            let mut reputacion = self.reputaciones
                .get(orden.vendedor)
                .unwrap_or_else(ReputacionData::new);
            reputacion.agregar_calificacion_vendedor(calificacion)?;
            self.reputaciones.insert(orden.vendedor, &reputacion);

            // Actualizar estadísticas de categoría
            if let Some(producto) = self.productos.get(orden.producto_id) {
                let mut stats = self.estadisticas_por_categoria
                    .get(&producto.categoria)
                    .unwrap_or((0, 0, 0));
                
                stats.0 = stats.0.checked_add(1).ok_or(ContractError::Overflow)?;
                stats.1 = stats.1.checked_add(calificacion as u128).ok_or(ContractError::Overflow)?;
                stats.2 = stats.2.checked_add(1).ok_or(ContractError::Overflow)?;
                
                self.estadisticas_por_categoria.insert(&producto.categoria, &stats);
            }

            Ok(())
        }

        /// El vendedor califica al comprador.
        fn _calificar_comprador(
            &mut self,
            vendedor: AccountId,
            orden_id: u128,
            calificacion: u8,
        ) -> Result<(), ContractError> {
            // Validar rango de calificación
            Self::_validar_calificacion(calificacion)?;

            // Obtener y validar la orden
            let orden = self.ordenes
                .get(orden_id)
                .ok_or(ContractError::OrdenNoExiste)?;

            // Verificar que el caller es el vendedor
            if orden.vendedor != vendedor {
                return Err(ContractError::NoAutorizado);
            }

            // Solo se puede calificar si la orden está recibida
            if orden.estado != EstadoOrden::Recibido {
                return Err(ContractError::OrdenNoRecibida);
            }

            // Obtener calificaciones existentes
            let mut calificaciones = self.calificaciones_por_orden
                .get(orden_id)
                .ok_or(ContractError::EstadoInvalido)?;

            // Verificar que no haya calificado antes
            if calificaciones.calificacion_vendedor.is_some() {
                return Err(ContractError::YaCalificado);
            }

            // Guardar la calificación
            calificaciones.calificacion_vendedor = Some(calificacion);
            self.calificaciones_por_orden.insert(orden_id, &calificaciones);

            // Actualizar reputación del comprador
            let mut reputacion = self.reputaciones
                .get(orden.comprador)
                .unwrap_or_else(ReputacionData::new);
            reputacion.agregar_calificacion_comprador(calificacion)?;
            self.reputaciones.insert(orden.comprador, &reputacion);

            Ok(())
        }

        /// Obtiene la reputación de un usuario.
        fn _obtener_reputacion(&self, usuario: AccountId) -> Option<ReputacionData> {
            self.reputaciones.get(usuario)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        fn default_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }

        fn init_contract() -> Marketplace {
            Marketplace::new()
        }

        #[ink::test]
        fn registrar_y_obtener_rol_funciona() {
            let accounts = default_accounts();
            let mut c = init_contract();
            assert_eq!(c._registrar_usuario(accounts.alice, Roles::Comprador), Ok(()));
            assert_eq!(c._obtener_rol(accounts.alice), Some(Roles::Comprador));
        }

        #[ink::test]
        fn no_se_puede_registrar_dos_veces() {
            let accounts = default_accounts();
            let mut c = init_contract();
            let _ = c._registrar_usuario(accounts.alice, Roles::Vendedor);
            assert_eq!(c._registrar_usuario(accounts.alice, Roles::Comprador), Err(ContractError::YaRegistrado));
        }

        #[ink::test]
        fn modificar_rol_funciona_y_falla_si_no_registrado() {
            let accounts = default_accounts();
            let mut c = init_contract();
            assert_eq!(c._modificar_rol(accounts.alice, Roles::Ambos), Err(ContractError::UsuarioNoRegistrado));
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            assert_eq!(c._modificar_rol(accounts.alice, Roles::Vendedor), Ok(()));
            assert_eq!(c._obtener_rol(accounts.alice), Some(Roles::Vendedor));
        }

        #[ink::test]
        fn publicar_producto_funciona() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            let id = c._publicar_producto(
                accounts.bob,
                "Camisa".into(),
                "Camisa de lino".into(),
                100,
                3,
                "Ropa".into()
            ).unwrap();
            assert_eq!(id, 1);
        }

        #[ink::test]
        fn publicar_producto_con_rol_invalido_falla() {
            let accounts = default_accounts();
            let mut c = init_contract();
            assert_eq!(
                c._publicar_producto(accounts.alice, "A".into(), "B".into(), 1, 1, "X".into()),
                Err(ContractError::NoVendedor)
            );
        }

        #[ink::test]
        fn crear_orden_funciona_y_valida_stock() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();

            c._publicar_producto(accounts.bob, "Libro".into(), "Rust".into(), 500, 5, "Libros".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 3).unwrap();
            assert_eq!(oid, 1);

            assert_eq!(c._crear_orden(accounts.alice, 1, 10), Err(ContractError::StockInsuficiente));
            assert_eq!(c._crear_orden(accounts.alice, 999, 1), Err(ContractError::ProductoNoEncontrado));
        }

        #[ink::test]
        fn crear_orden_por_usuario_no_autorizado_falla() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._publicar_producto(accounts.bob, "Item".into(), "Desc".into(), 1, 1, "C".into()).unwrap();
            assert_eq!(
                c._crear_orden(accounts.charlie, 1, 1),
                Err(ContractError::NoAutorizado)
            );
        }

        #[ink::test]
        fn orden_estado_transiciones_correctas() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();

            c._publicar_producto(accounts.bob, "Mouse".into(), "Gaming".into(), 200, 2, "Perifericos".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();

            assert_eq!(c._marcar_enviada(accounts.bob, oid), Ok(()));
            assert_eq!(c._marcar_recibida(accounts.alice, oid), Ok(()));
        }

        #[ink::test]
        fn estado_invalido_en_transiciones() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();

            c._publicar_producto(accounts.bob, "K".into(), "J".into(), 2, 2, "Z".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();

            c._marcar_enviada(accounts.bob, oid).unwrap();
            assert_eq!(c._marcar_enviada(accounts.bob, oid), Err(ContractError::EstadoInvalido));
            assert_eq!(c._marcar_recibida(accounts.bob, oid), Err(ContractError::NoAutorizado));
            assert_eq!(c._marcar_recibida(accounts.alice, oid), Ok(()));
            assert_eq!(c._marcar_recibida(accounts.alice, oid), Err(ContractError::EstadoInvalido));
        }

        #[ink::test]
        fn cancelacion_mutua_funciona() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();

            c._publicar_producto(accounts.bob, "Café".into(), "Molido".into(), 100, 1, "Alimentos".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();

            assert_eq!(c._solicitar_cancel_comprador(accounts.alice, oid), Ok(()));
            assert_eq!(c._aceptar_cancel_vendedor(accounts.bob, oid), Ok(()));

            let orden = c.ordenes.get(oid).unwrap();
            assert_eq!(orden.estado, EstadoOrden::Cancelada);
        }

        #[ink::test]
        fn solo_uno_cancela_no_avanza_estado() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();

            c._publicar_producto(accounts.bob, "Mate".into(), "Dulce".into(), 100, 1, "Bebidas".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();

            c._solicitar_cancel_comprador(accounts.alice, oid).unwrap();
            let orden = c.ordenes.get(oid).unwrap();
            assert_eq!(orden.estado, EstadoOrden::Pendiente);
        }

        #[ink::test]
        fn acciones_sobre_orden_inexistente_fallan() {
            let accounts = default_accounts();
            let mut c = init_contract();
            assert_eq!(c._marcar_enviada(accounts.bob, 123), Err(ContractError::OrdenNoExiste));
            assert_eq!(c._marcar_recibida(accounts.alice, 123), Err(ContractError::OrdenNoExiste));
            assert_eq!(c._aceptar_cancel_vendedor(accounts.bob, 123), Err(ContractError::OrdenNoExiste));
            assert_eq!(c._solicitar_cancel_comprador(accounts.alice, 123), Err(ContractError::OrdenNoExiste));
        }

        #[ink::test]
        fn ver_mis_productos_y_todos_funciona() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._publicar_producto(accounts.bob, "A".into(), "B".into(), 1, 1, "X".into()).unwrap();
            c._publicar_producto(accounts.bob, "C".into(), "D".into(), 1, 1, "Y".into()).unwrap();

            let personales = c._ver_mis_productos(accounts.bob);
            assert_eq!(personales.len(), 2);

            let todos = c._ver_todos_los_productos();
            assert_eq!(todos.len(), 2);
        }

        #[ink::test]
        fn overflow_en_productos_y_ordenes() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c.siguiente_producto_id = u128::MAX;
            c.siguiente_orden_id = u128::MAX;
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            let prod = c._publicar_producto(accounts.bob, "Z".into(), "Z".into(), 1, 1, "Z".into());
            assert_eq!(prod, Err(ContractError::Overflow));

            c.siguiente_producto_id = 1;
            c._publicar_producto(accounts.bob, "A".into(), "B".into(), 1, 1, "C".into()).unwrap();
            let orden = c._crear_orden(accounts.alice, 1, 1);
            assert_eq!(orden, Err(ContractError::Overflow));
        }

        // ===== Tests for public message dispatchers =====
        #[ink::test]
        fn public_registrar_usuario_mensaje() {
            let accounts = default_accounts();
            let mut c = init_contract();
            // Alice registers
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            assert_eq!(c.registrar_usuario(Roles::Comprador), Ok(()));
            // Alice duplicate registration
            assert_eq!(c.registrar_usuario(Roles::Vendedor), Err(ContractError::YaRegistrado));
        }

        #[ink::test]
        fn public_modificar_rol_mensaje() {
            let accounts = default_accounts();
            let mut c = init_contract();
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            assert_eq!(c.modificar_rol(Roles::Vendedor), Err(ContractError::UsuarioNoRegistrado));
            // Register and modify
            assert_eq!(c.registrar_usuario(Roles::Ambos), Ok(()));
            assert_eq!(c.modificar_rol(Roles::Vendedor), Ok(()));
        }

        #[ink::test]
        fn public_obtener_rol_mensaje() {
            let accounts = default_accounts();
            let mut c = init_contract();
            // None
            assert_eq!(c.obtener_rol(accounts.charlie), None);
            // After registration
            test::set_caller::<DefaultEnvironment>(accounts.charlie);
            c.registrar_usuario(Roles::Ambos).unwrap();
            assert_eq!(c.obtener_rol(accounts.charlie), Some(Roles::Ambos));
        }

        #[ink::test]
        fn public_publicar_producto_mensaje() {
            let accounts = default_accounts();
            let mut c = init_contract();
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            // Fail without role
            assert_eq!(c.publicar_producto("X".into(), "Y".into(), 10,1,"C".into()), Err(ContractError::NoVendedor));
            // Register and publish
            c.registrar_usuario(Roles::Vendedor).unwrap();
            let pid = c.publicar_producto("X".into(),"Y".into(),10,1,"C".into()).unwrap();
            assert_eq!(pid, 1);
        }

        #[ink::test]
        fn public_ver_productos_mensaje() {
            let accounts = default_accounts();
            let mut c = init_contract();
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            c.registrar_usuario(Roles::Vendedor).unwrap();
            c.publicar_producto("A".into(),"B".into(),1,1,"Cat".into()).unwrap();
            // ver_mis
            let own = c.ver_mis_productos();
            assert_eq!(own.len(),1);
            // ver_todos
            let all = c.ver_todos_los_productos();
            assert_eq!(all.len(),1);
        }

        #[ink::test]
        fn public_crear_orden_de_compra_mensaje() {
            let accounts = default_accounts();
            let mut c = init_contract();
            // setup
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            c.registrar_usuario(Roles::Vendedor).unwrap();
            let pid = c.publicar_producto("P".into(),"D".into(),5,2,"Cat".into()).unwrap();
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            c.registrar_usuario(Roles::Comprador).unwrap();
            let oid = c.crear_orden_de_compra(pid,1).unwrap();
            assert_eq!(oid,1);
        }

        #[ink::test]
        fn public_marcar_enviada_recibida_mensajes() {
            let accounts = default_accounts();
            let mut c = init_contract();
            // prepare
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            c.registrar_usuario(Roles::Vendedor).unwrap();
            let pid = c.publicar_producto("P".into(),"D".into(),5,1,"Cat".into()).unwrap();
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            c.registrar_usuario(Roles::Comprador).unwrap();
            let oid = c.crear_orden_de_compra(pid,1).unwrap();
            // send
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            assert_eq!(c.marcar_orden_enviada(oid), Ok(()));
            // receive
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            assert_eq!(c.marcar_orden_recibida(oid), Ok(()));
        }

        #[ink::test]
        fn public_cancelacion_mensajes() {
            let accounts = default_accounts();
            let mut c = init_contract();
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            c.registrar_usuario(Roles::Vendedor).unwrap();
            let pid = c.publicar_producto("C".into(),"D".into(),5,1,"Cat".into()).unwrap();
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            c.registrar_usuario(Roles::Comprador).unwrap();
            let oid = c.crear_orden_de_compra(pid,1).unwrap();
            // comprador solicita
            assert_eq!(c.comprador_solicita_cancelacion(oid), Ok(()));
            // vendedor acepta
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            assert_eq!(c.vendedor_acepta_cancelacion(oid), Ok(()));
        }

        // ===== Tests adicionales solicitados =====

        #[ink::test]
        fn registrar_usuario_como_vendedor() {
            let accounts = default_accounts();
            let mut c = init_contract();
            assert_eq!(c._registrar_usuario(accounts.alice, Roles::Vendedor), Ok(()));
            assert_eq!(c._obtener_rol(accounts.alice), Some(Roles::Vendedor));
        }

        #[ink::test]
        fn registrar_usuario_como_ambos() {
            let accounts = default_accounts();
            let mut c = init_contract();
            assert_eq!(c._registrar_usuario(accounts.alice, Roles::Ambos), Ok(()));
            assert_eq!(c._obtener_rol(accounts.alice), Some(Roles::Ambos));
        }

        #[ink::test]
        fn modificar_rol_solo_permite_agregar_no_quitar() {
            let accounts = default_accounts();
            let mut c = init_contract();
            // Registrar como Comprador y agregar Vendedor -> Ambos
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            assert_eq!(c._modificar_rol(accounts.alice, Roles::Vendedor), Ok(()));
            assert_eq!(c._obtener_rol(accounts.alice), Some(Roles::Ambos));
            
            // Registrar como Vendedor y agregar Comprador -> Ambos
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            assert_eq!(c._modificar_rol(accounts.bob, Roles::Comprador), Ok(()));
            assert_eq!(c._obtener_rol(accounts.bob), Some(Roles::Ambos));
            
            // Si ya es Ambos, intentar cambiar a Vendedor mantiene Ambos (no quita Comprador)
            c._registrar_usuario(accounts.charlie, Roles::Ambos).unwrap();
            assert_eq!(c._modificar_rol(accounts.charlie, Roles::Vendedor), Ok(()));
            assert_eq!(c._obtener_rol(accounts.charlie), Some(Roles::Ambos));
        }

        #[ink::test]
        fn no_se_puede_publicar_con_nombre_vacio() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Vendedor).unwrap();
            assert_eq!(
                c._publicar_producto(accounts.alice, "".into(), "Desc".into(), 100, 1, "Cat".into()),
                Err(ContractError::DatosInvalidos)
            );
        }

        #[ink::test]
        fn no_se_puede_publicar_con_descripcion_vacia() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Vendedor).unwrap();
            assert_eq!(
                c._publicar_producto(accounts.alice, "Nombre".into(), "".into(), 100, 1, "Cat".into()),
                Err(ContractError::DatosInvalidos)
            );
        }

        #[ink::test]
        fn no_se_puede_publicar_con_categoria_vacia() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Vendedor).unwrap();
            assert_eq!(
                c._publicar_producto(accounts.alice, "Nombre".into(), "Desc".into(), 100, 1, "".into()),
                Err(ContractError::DatosInvalidos)
            );
        }

        #[ink::test]
        fn no_se_puede_publicar_con_precio_cero() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Vendedor).unwrap();
            assert_eq!(
                c._publicar_producto(accounts.alice, "Nombre".into(), "Desc".into(), 0, 1, "Cat".into()),
                Err(ContractError::DatosInvalidos)
            );
        }

        #[ink::test]
        fn no_se_puede_publicar_con_cantidad_cero() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Vendedor).unwrap();
            assert_eq!(
                c._publicar_producto(accounts.alice, "Nombre".into(), "Desc".into(), 100, 0, "Cat".into()),
                Err(ContractError::DatosInvalidos)
            );
        }

        #[ink::test]
        fn no_se_puede_crear_orden_con_cantidad_cero() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._publicar_producto(accounts.bob, "Producto".into(), "Desc".into(), 100, 5, "Cat".into()).unwrap();
            
            assert_eq!(
                c._crear_orden(accounts.alice, 1, 0),
                Err(ContractError::StockInsuficiente)
            );
        }

        #[ink::test]
        fn no_se_puede_marcar_orden_recibida_desde_pendiente() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._publicar_producto(accounts.bob, "Producto".into(), "Desc".into(), 100, 5, "Cat".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();

            // No se puede pasar directamente de Pendiente a Recibido
            assert_eq!(
                c._marcar_recibida(accounts.alice, oid),
                Err(ContractError::EstadoInvalido)
            );
        }

        #[ink::test]
        fn no_se_puede_retroceder_de_recibido_a_enviado() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._publicar_producto(accounts.bob, "Producto".into(), "Desc".into(), 100, 5, "Cat".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();

            c._marcar_enviada(accounts.bob, oid).unwrap();
            c._marcar_recibida(accounts.alice, oid).unwrap();

            // No se puede retroceder de Recibido a Enviado
            assert_eq!(
                c._marcar_enviada(accounts.bob, oid),
                Err(ContractError::EstadoInvalido)
            );
        }

        #[ink::test]
        fn no_se_puede_retroceder_de_recibido_a_pendiente() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._publicar_producto(accounts.bob, "Producto".into(), "Desc".into(), 100, 5, "Cat".into()).unwrap();
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();

            c._marcar_enviada(accounts.bob, oid).unwrap();
            c._marcar_recibida(accounts.alice, oid).unwrap();

            // Verificar que el estado es Recibido
            let orden = c.ordenes.get(oid).unwrap();
            assert_eq!(orden.estado, EstadoOrden::Recibido);
        }

        #[ink::test]
        fn cancelacion_devuelve_stock() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            // Publicar producto con stock 5
            let pid = c._publicar_producto(
                accounts.bob, 
                "Producto".into(), 
                "Desc".into(), 
                100, 
                5, 
                "Cat".into()
            ).unwrap();
            
            // Verificar stock inicial
            let producto_antes = c.productos.get(pid).unwrap();
            assert_eq!(producto_antes.cantidad, 5);

            // Crear orden de 2 unidades
            let oid = c._crear_orden(accounts.alice, pid, 2).unwrap();
            
            // Verificar que el stock se redujo
            let producto_despues = c.productos.get(pid).unwrap();
            assert_eq!(producto_despues.cantidad, 3);

            // Cancelar la orden
            c._solicitar_cancel_comprador(accounts.alice, oid).unwrap();
            c._aceptar_cancel_vendedor(accounts.bob, oid).unwrap();

            // Verificar que el stock se devolvió
            let producto_final = c.productos.get(pid).unwrap();
            assert_eq!(producto_final.cantidad, 5);
        }

        #[ink::test]
        fn obtener_estado_orden_funciona() {
            let accounts = default_accounts();
            let mut c = init_contract();
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._publicar_producto(accounts.bob, "Producto".into(), "Desc".into(), 100, 5, "Cat".into()).unwrap();
            
            let oid = c._crear_orden(accounts.alice, 1, 1).unwrap();
            
            // Estado inicial: Pendiente
            assert_eq!(c.obtener_estado_orden(oid), Some(EstadoOrden::Pendiente));
            
            // Estado después de marcar como enviada: Enviado
            c._marcar_enviada(accounts.bob, oid).unwrap();
            assert_eq!(c.obtener_estado_orden(oid), Some(EstadoOrden::Enviado));
            
            // Estado después de marcar como recibida: Recibido
            c._marcar_recibida(accounts.alice, oid).unwrap();
            assert_eq!(c.obtener_estado_orden(oid), Some(EstadoOrden::Recibido));
        }

        #[ink::test]
        fn obtener_estado_orden_inexistente() {
            let c = init_contract();
            assert_eq!(c.obtener_estado_orden(999), None);
        }

        // ===== Tests de sistema de reputación =====

        #[ink::test]
        fn comprador_califica_vendedor_funciona() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            let pid = c._publicar_producto(
                accounts.bob,
                "Producto".into(),
                "Desc".into(),
                100,
                5,
                "Cat".into()
            ).unwrap();
            
            let oid = c._crear_orden(accounts.alice, pid, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid).unwrap();
            c._marcar_recibida(accounts.alice, oid).unwrap();
            
            // Comprador califica al vendedor
            assert_eq!(c.comprador_califica_vendedor(oid, 5), Ok(()));
            
            // Verificar que la calificación se guardó
            let calificaciones = c.obtener_calificaciones_orden(oid).unwrap();
            assert_eq!(calificaciones.calificacion_comprador, Some(5));
            assert_eq!(calificaciones.calificacion_vendedor, None);
            
            // Verificar reputación del vendedor
            let reputacion = c.obtener_reputacion(accounts.bob).unwrap();
            assert_eq!(reputacion.promedio_vendedor(), Some(5));
        }

        #[ink::test]
        fn vendedor_califica_comprador_funciona() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            let pid = c._publicar_producto(
                accounts.bob,
                "Producto".into(),
                "Desc".into(),
                100,
                5,
                "Cat".into()
            ).unwrap();
            
            let oid = c._crear_orden(accounts.alice, pid, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid).unwrap();
            c._marcar_recibida(accounts.alice, oid).unwrap();
            
            // Vendedor califica al comprador
            assert_eq!(c.vendedor_califica_comprador(oid, 4), Ok(()));
            
            // Verificar que la calificación se guardó
            let calificaciones = c.obtener_calificaciones_orden(oid).unwrap();
            assert_eq!(calificaciones.calificacion_comprador, None);
            assert_eq!(calificaciones.calificacion_vendedor, Some(4));
            
            // Verificar reputación del comprador
            let reputacion = c.obtener_reputacion(accounts.alice).unwrap();
            assert_eq!(reputacion.promedio_comprador(), Some(4));
        }

        #[ink::test]
        fn no_se_puede_calificar_si_orden_no_recibida() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            let pid = c._publicar_producto(
                accounts.bob,
                "Producto".into(),
                "Desc".into(),
                100,
                5,
                "Cat".into()
            ).unwrap();
            
            let oid = c._crear_orden(accounts.alice, pid, 1).unwrap();
            
            // No se puede calificar si la orden está pendiente
            assert_eq!(
                c.comprador_califica_vendedor(oid, 5),
                Err(ContractError::OrdenNoRecibida)
            );
        }

        #[ink::test]
        fn no_se_puede_calificar_dos_veces() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            let pid = c._publicar_producto(
                accounts.bob,
                "Producto".into(),
                "Desc".into(),
                100,
                5,
                "Cat".into()
            ).unwrap();
            
            let oid = c._crear_orden(accounts.alice, pid, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid).unwrap();
            c._marcar_recibida(accounts.alice, oid).unwrap();
            
            // Primera calificación OK
            assert_eq!(c.comprador_califica_vendedor(oid, 5), Ok(()));
            
            // Segunda calificación debe fallar
            assert_eq!(
                c.comprador_califica_vendedor(oid, 4),
                Err(ContractError::YaCalificado)
            );
        }

        #[ink::test]
        fn calificacion_invalida_fuera_de_rango() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            let pid = c._publicar_producto(
                accounts.bob,
                "Producto".into(),
                "Desc".into(),
                100,
                5,
                "Cat".into()
            ).unwrap();
            
            let oid = c._crear_orden(accounts.alice, pid, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid).unwrap();
            c._marcar_recibida(accounts.alice, oid).unwrap();
            
            // Calificación 0 (inválida)
            assert_eq!(
                c.comprador_califica_vendedor(oid, 0),
                Err(ContractError::CalificacionInvalida)
            );
            
            // Calificación 6 (inválida)
            assert_eq!(
                c.comprador_califica_vendedor(oid, 6),
                Err(ContractError::CalificacionInvalida)
            );
        }

        #[ink::test]
        fn reputacion_acumulada_multiple_calificaciones() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            c._registrar_usuario(accounts.charlie, Roles::Comprador).unwrap();
            
            // Primera orden
            let pid1 = c._publicar_producto(
                accounts.bob,
                "Producto1".into(),
                "Desc".into(),
                100,
                10,
                "Cat1".into()
            ).unwrap();
            let oid1 = c._crear_orden(accounts.alice, pid1, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid1).unwrap();
            c._marcar_recibida(accounts.alice, oid1).unwrap();
            c.comprador_califica_vendedor(oid1, 5).unwrap();
            
            // Segunda orden
            let pid2 = c._publicar_producto(
                accounts.bob,
                "Producto2".into(),
                "Desc".into(),
                100,
                10,
                "Cat1".into()
            ).unwrap();
            let oid2 = c._crear_orden(accounts.charlie, pid2, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid2).unwrap();
            c._marcar_recibida(accounts.charlie, oid2).unwrap();
            c.comprador_califica_vendedor(oid2, 3).unwrap();
            
            // Verificar promedio: (5 + 3) / 2 = 4
            let reputacion = c.obtener_reputacion(accounts.bob).unwrap();
            assert_eq!(reputacion.promedio_vendedor(), Some(4));
            assert_eq!(reputacion.total_calificaciones_vendedor, 2);
        }

        #[ink::test]
        fn obtener_reputacion_como_comprador_y_vendedor() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Ambos).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Ambos).unwrap();
            
            // Alice como vendedor
            let pid = c._publicar_producto(
                accounts.alice,
                "Producto".into(),
                "Desc".into(),
                100,
                5,
                "Cat".into()
            ).unwrap();
            
            let oid = c._crear_orden(accounts.bob, pid, 1).unwrap();
            c._marcar_enviada(accounts.alice, oid).unwrap();
            c._marcar_recibida(accounts.bob, oid).unwrap();
            
            c.comprador_califica_vendedor(oid, 5).unwrap();
            c.vendedor_califica_comprador(oid, 4).unwrap();
            
            // Alice tiene reputación como vendedor
            let reputacion_alice = c.obtener_reputacion(accounts.alice).unwrap();
            assert_eq!(reputacion_alice.promedio_vendedor(), Some(5));
            
            // Bob tiene reputación como comprador
            let reputacion_bob = c.obtener_reputacion(accounts.bob).unwrap();
            assert_eq!(reputacion_bob.promedio_comprador(), Some(4));
        }

        #[ink::test]
        fn ventas_por_producto_se_registran() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            let pid = c._publicar_producto(
                accounts.bob,
                "Producto".into(),
                "Desc".into(),
                100,
                10,
                "Cat".into()
            ).unwrap();
            
            // Antes de recibir, no hay ventas registradas
            assert_eq!(c.obtener_ventas_producto(pid), 0);
            
            let oid = c._crear_orden(accounts.alice, pid, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid).unwrap();
            
            // Todavía no hay ventas (orden no recibida)
            assert_eq!(c.obtener_ventas_producto(pid), 0);
            
            c._marcar_recibida(accounts.alice, oid).unwrap();
            
            // Ahora sí hay una venta registrada
            assert_eq!(c.obtener_ventas_producto(pid), 1);
        }

        #[ink::test]
        fn estadisticas_por_categoria() {
            let accounts = default_accounts();
            let mut c = init_contract();
            
            c._registrar_usuario(accounts.alice, Roles::Comprador).unwrap();
            c._registrar_usuario(accounts.bob, Roles::Vendedor).unwrap();
            
            let pid = c._publicar_producto(
                accounts.bob,
                "Producto".into(),
                "Desc".into(),
                100,
                5,
                "Electronica".into()
            ).unwrap();
            
            let oid = c._crear_orden(accounts.alice, pid, 1).unwrap();
            c._marcar_enviada(accounts.bob, oid).unwrap();
            c._marcar_recibida(accounts.alice, oid).unwrap();
            
            c.comprador_califica_vendedor(oid, 5).unwrap();
            
            // Verificar estadísticas de la categoría
            let stats = c.obtener_estadisticas_categoria("Electronica".into()).unwrap();
            assert_eq!(stats.0, 1); // total_ventas
            assert_eq!(stats.1, 5); // suma_calificaciones
            assert_eq!(stats.2, 1); // cantidad_calificaciones
        }
    }
}
