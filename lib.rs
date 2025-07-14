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
    pub enum Roles { Comprador, Vendedor, Ambos }

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


        // Funciones privadas
        fn _registrar_usuario(&mut self, caller: AccountId, rol: Roles) -> Result<(), ContractError> {
            if self.roles.contains(caller) { return Err(ContractError::YaRegistrado) }
            self.roles.insert(caller, &rol); Ok(())
        }
        fn _modificar_rol(&mut self, caller: AccountId, nuevo_rol: Roles) -> Result<(), ContractError> {
            if !self.roles.contains(caller) { return Err(ContractError::UsuarioNoRegistrado) }
            self.roles.insert(caller, &nuevo_rol); Ok(())
        }
        fn _obtener_rol(&self, usuario: AccountId) -> Option<Roles> { self.roles.get(usuario) }
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
            if !matches!(rol, Some(Roles::Vendedor) | Some(Roles::Ambos)) { return Err(ContractError::NoVendedor) }
            let pid = self.siguiente_producto_id;
            let producto = Producto { nombre, descripcion, precio, cantidad, categoria, vendedor: caller };
            self.productos.insert(pid, &producto);
            let mut lista = self.productos_por_usuario.get(&caller).unwrap_or_default();
            lista.push(pid);
            self.productos_por_usuario.insert(&caller, &lista);
            self.siguiente_producto_id = pid.checked_add(1).ok_or(ContractError::Overflow)?;
            Ok(pid)
        }

        fn _ver_mis_productos(&self, caller: AccountId) -> Vec<(u128, Producto)> {
            self.productos_por_usuario.get(&caller).unwrap_or_default()
                .into_iter().filter_map(|id| self.productos.get(id).map(|p|(id,p))).collect()
        }

        fn _ver_todos_los_productos(&self) -> Vec<(u128, Producto)> {
            let mut acc = Vec::new();
            for id in 1..self.siguiente_producto_id { if let Some(p)=self.productos.get(id){acc.push((id,p));}} acc
        }

        fn _crear_orden(&mut self, comprador: AccountId, producto_id: u128, cantidad: u32) -> Result<u128, ContractError> {
            let rol = self.roles.get(&comprador);
            if !matches!(rol, Some(Roles::Comprador)|Some(Roles::Ambos)){return Err(ContractError::NoAutorizado)}
            let mut producto = self.productos.get(producto_id).ok_or(ContractError::ProductoNoEncontrado)?;
            if producto.cantidad < cantidad {return Err(ContractError::StockInsuficiente)}
            producto.cantidad = producto.cantidad.checked_sub(cantidad).ok_or(ContractError::Overflow)?;
            self.productos.insert(producto_id,&producto);
            let oid=self.siguiente_orden_id;
            let orden=Orden{comprador,vendedor:producto.vendedor,producto_id,cantidad,estado:EstadoOrden::Pendiente,comprador_acepta_cancelar:false,vendedor_acepta_cancelar:false};
            self.ordenes.insert(oid,&orden);
            let mut h=self.ordenes_por_usuario.get(&comprador).unwrap_or_default(); h.push(oid);
            self.ordenes_por_usuario.insert(&comprador,&h);
            self.siguiente_orden_id=oid.checked_add(1).ok_or(ContractError::Overflow)?;
            Ok(oid)
        }

        fn _marcar_enviada(&mut self, caller: AccountId, orden_id: u128)->Result<(),ContractError>{
            let mut o=self.ordenes.get(orden_id).ok_or(ContractError::OrdenNoExiste)?;
            if o.vendedor!=caller{return Err(ContractError::NoAutorizado)}
            if o.estado!=EstadoOrden::Pendiente{return Err(ContractError::EstadoInvalido)}
            o.estado=EstadoOrden::Enviado;self.ordenes.insert(orden_id,&o);Ok(())
        }

        fn _marcar_recibida(&mut self, caller: AccountId, orden_id: u128)->Result<(),ContractError>{
            let mut o=self.ordenes.get(orden_id).ok_or(ContractError::OrdenNoExiste)?;
            if o.comprador!=caller{return Err(ContractError::NoAutorizado)}
            if o.estado!=EstadoOrden::Enviado{return Err(ContractError::EstadoInvalido)}
            o.estado=EstadoOrden::Recibido;self.ordenes.insert(orden_id,&o);Ok(())
        }

        fn _solicitar_cancel_comprador(&mut self, caller: AccountId, orden_id: u128)->Result<(),ContractError>{
            let mut o=self.ordenes.get(orden_id).ok_or(ContractError::OrdenNoExiste)?;
            if o.comprador!=caller{return Err(ContractError::NoAutorizado)}
            if o.estado!=EstadoOrden::Pendiente{return Err(ContractError::EstadoInvalido)}
            o.comprador_acepta_cancelar=true; if o.vendedor_acepta_cancelar{o.estado=EstadoOrden::Cancelada}; self.ordenes.insert(orden_id,&o);Ok(())
        }

        fn _aceptar_cancel_vendedor(&mut self, caller: AccountId, orden_id: u128)->Result<(),ContractError>{
            let mut o=self.ordenes.get(orden_id).ok_or(ContractError::OrdenNoExiste)?;
            if o.vendedor!=caller{return Err(ContractError::NoAutorizado)}
            if o.estado!=EstadoOrden::Pendiente{return Err(ContractError::EstadoInvalido)}
            o.vendedor_acepta_cancelar=true; if o.comprador_acepta_cancelar{o.estado=EstadoOrden::Cancelada}; self.ordenes.insert(orden_id,&o);Ok(())
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
}



}


