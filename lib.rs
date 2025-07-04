#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace {
    
    use ink::storage::Mapping;


    #[derive(Clone)]
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

    #[ink(storage)]
    pub struct RegistroUsuarios {
        roles: Mapping<AccountId, Roles>,
    }

    impl RegistroUsuarios {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                roles: Mapping::default(),
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
    }


}
