#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;


    #[derive(Clone,PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub enum Roles {
        Comprador,
        Vendedor,
        Ambos,
    }


    #[derive(Clone,PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub enum EstadoOrden {
        Pendiente,
        Enviado,
        Recibido,
        Cancelada,
    }

    #[derive(Clone)]
    // error custom 
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
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

    #[derive(Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub struct Producto{
        pub nombre:String,
        pub descripcion:String,
        pub precio: u128,
        pub cantidad: u32,
        pub categoria:String,
        pub vendedor: AccountId,
    }

    #[derive(Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub struct Orden {
        pub comprador: AccountId,
        pub vendedor: AccountId,
        pub producto_id: u128,
        pub cantidad: u32,
        pub estado: EstadoOrden,
        pub comprador_acepta_cancelar: bool,
        pub vendedor_acepta_cancelar: bool,
    }


    #[ink(storage)]
    pub struct Marketplace {
        roles: Mapping<AccountId, Roles>,

        productos: Mapping<u128, Producto>,
        productos_por_usuario: Mapping<AccountId, Vec<u128>>,
        siguiente_producto_id: u128,

        ordenes: Mapping<u128, Orden>,
        ordenes_por_usuario: Mapping<AccountId, Vec<u128>>,
        siguiente_orden_id: u128,
    }

    impl Marketplace{ 
        // SISTEMA DE GESTION DE USUARIOS
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                roles: Mapping::default(),
                productos: Mapping::default(), 
                productos_por_usuario: Mapping::default(), 
                siguiente_producto_id:1,
                ordenes: Mapping::default(),
                ordenes_por_usuario: Mapping::default(),
                siguiente_orden_id: 1,
            }
        }

        #[ink(message)]
        pub fn registrar_usuario(&mut self, rol: Roles) -> Result<(), ContractError> {
            let caller = self.env().caller();
            if self.roles.contains(caller) {
                return Err(ContractError::YaRegistrado);
            }
            self.roles.insert(caller, &rol);
            Ok(())
        }

        #[ink(message)]
        pub fn modificar_rol(&mut self, nuevo_rol: Roles) ->   Result<(), ContractError> {
            let caller = self.env().caller();
            if !self.roles.contains(caller) {
                return Err(ContractError::UsuarioNoRegistrado);
            }
            self.roles.insert(caller, &nuevo_rol);
            Ok(())
        }

        #[ink(message)]
        pub fn obtener_rol(&self, usuario: AccountId) -> Option<Roles> {
            self.roles.get(usuario)
        }

        // PUBLICACION DE PRODUCTOS
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

            let rol = self.roles.get(&caller);

            if !matches!(rol, Some(Roles::Vendedor) | Some(Roles::Ambos)) {
                return Err(ContractError::NoVendedor);
            }

            let producto_id = self.siguiente_producto_id;

            let producto = Producto {
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
                vendedor: caller,   
            };

            self.productos.insert(producto_id, &producto);

            let mut productos_usuario = self.productos_por_usuario.get(&caller).unwrap_or_default();
            productos_usuario.push(producto_id);

            self.productos_por_usuario.insert(&caller, &productos_usuario);
            self.siguiente_producto_id = producto_id
                .checked_add(1)
                .ok_or(ContractError::Overflow)?;
            Ok(producto_id)

        }


        #[ink(message)]
        pub fn ver_mis_productos(&self) -> Vec<(u128, Producto)> {
            let caller = self.env().caller();
            let productos = self
                .productos_por_usuario
                .get(&caller)
                .unwrap_or_default();

            productos.into_iter()
                .filter_map(|id| {
                    self.productos.get(id).map(|p| (id, p))
                })
                .collect()
        }


        #[ink(message)]
        pub fn ver_todos_los_productos(&self) -> Vec<(u128, Producto)> {
            let mut productos = Vec::new();

            for id in 1..self.siguiente_producto_id {
                if let Some(producto) = self.productos.get(id) {
                    productos.push((id, producto));
                }
            }

            productos
        }


        // ORDENES DE COMPRA

        #[ink(message)]
        pub fn crear_orden_de_compra(&mut self, producto_id: u128, cantidad: u32, ) -> Result<u128, ContractError> {

            let comprador = self.env().caller();

            let rol = self.roles.get(&comprador);
            if !matches!(rol, Some(Roles::Comprador) | Some(Roles::Ambos)) {
                return Err(ContractError::NoAutorizado);
            }

            let mut producto = self.productos.get(producto_id).ok_or(ContractError::ProductoNoEncontrado)?;
            if producto.cantidad < cantidad {
                return Err(ContractError::StockInsuficiente);
            }

            producto.cantidad = producto.cantidad.checked_sub(cantidad).ok_or(ContractError::Overflow)?;
            
            self.productos.insert(producto_id, &producto); // ACTUALIZO EL MAPPING DE PRODUCTOS

            let orden_id = self.siguiente_orden_id;
            let orden = Orden {
                comprador,
                vendedor: producto.vendedor,
                producto_id: producto_id,
                cantidad,
                estado: EstadoOrden::Pendiente,
                comprador_acepta_cancelar: false,
                vendedor_acepta_cancelar: false,
            };

            self.ordenes.insert(orden_id, &orden); 

            let mut ordenes_usuario = self.ordenes_por_usuario.get(&comprador).unwrap_or_default();
            ordenes_usuario.push(orden_id);
            self.ordenes_por_usuario.insert(&comprador, &ordenes_usuario);


            self.siguiente_orden_id = orden_id
                .checked_add(1)
                .ok_or(ContractError::Overflow)?;
            Ok(orden_id)
        }



    
    }



}
