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

    #[derive(Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub enum ContractError {
        YaRegistrado,
        UsuarioNoRegistrado,
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
    }


    #[ink(storage)]
    pub struct RegistroUsuarios {
        roles: Mapping<AccountId, Roles>,
        productos: Mapping<AccountId, Vec<Producto>>,
    }

    impl RegistroUsuarios {
        // SISTEMA DE GESTION DE USUARIOS
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                roles: Mapping::default(),
                productos: Mapping::default(), 
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
        pub fn modificar_rol(&mut self, nuevo_rol: Roles) -> Result<(), ContractError> {
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
        ) {
            let caller = self.env().caller();

            // Verificamos si el usuario tiene rol de Vendedor
            let rol = self.roles.get(&caller);
            assert!(
                matches!(rol, Some(Roles::Vendedor) | Some(Roles::Ambos)),
                "No sos vendedor"
            );
            let producto = Producto {
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
            };

            let mut lista = self.productos.get(&caller).unwrap_or_default();
            lista.push(producto);
            self.productos.insert(&caller, &lista);
        }


        #[ink(message)]
        pub fn ver_mis_productos(&self) -> Vec<Producto> {
            let caller = self.env().caller();
            self.productos.get(&caller).unwrap_or_default()
        }

        
    }



}
