#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
/// Contrato de reportes de solo lectura para consultar estadísticas del marketplace.
mod reportes_view {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::prelude::{string::String, vec::Vec};
    use ink::prelude::collections::BTreeMap;

    /// Tipo para representar un producto (debe coincidir con el del contrato Marketplace).
    #[derive(Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Producto {
        pub nombre: String,
        pub descripcion: String,
        pub precio: u128,
        pub cantidad: u32,
        pub categoria: String,
        pub vendedor: AccountId,
    }

    /// Tipo para representar datos de reputación (debe coincidir con el del contrato Marketplace).
    #[derive(Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ReputacionData {
        pub total_calificaciones_comprador: u32,
        pub suma_calificaciones_comprador: u128,
        pub total_calificaciones_vendedor: u32,
        pub suma_calificaciones_vendedor: u128,
    }

    impl ReputacionData {
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
    }

    /// Enum para errores del contrato de reportes.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ReportesError {
        LlamadaFallida,
        MarketplaceNoConfigurado,
    }

    /// Contrato de solo lectura para consultar estadísticas y reportes del marketplace.
    #[ink(storage)]
    pub struct ReportesView {
        /// Referencia al contrato principal del marketplace.
        marketplace: AccountId,
    }

    impl ReportesView {
        /// Crea una nueva instancia del contrato de reportes.
        /// Requiere la dirección (AccountId) del contrato principal del marketplace.
        #[ink(constructor)]
        pub fn new(marketplace: AccountId) -> Self {
            Self { marketplace }
        }

        /// Actualiza la dirección del contrato marketplace (útil para tests y migraciones).
        /// En producción, esto podría requerir permisos especiales.
        #[ink(message)]
        pub fn actualizar_marketplace(&mut self, nuevo_marketplace: AccountId) {
            self.marketplace = nuevo_marketplace;
        }

        /// Obtiene la dirección del contrato marketplace configurado.
        #[ink(message)]
        pub fn obtener_marketplace(&self) -> AccountId {
            self.marketplace
        }

        /// Obtiene el top 5 de vendedores con mejor reputación.
        /// Retorna un vector de tuplas (AccountId, promedio_reputacion).
        #[ink(message)]
        pub fn top_5_vendedores(&self) -> Vec<(AccountId, u128)> {
            self._obtener_top_vendedores(5)
        }

        /// Obtiene el top 5 de compradores con mejor reputación.
        /// Retorna un vector de tuplas (AccountId, promedio_reputacion).
        #[ink(message)]
        pub fn top_5_compradores(&self) -> Vec<(AccountId, u128)> {
            self._obtener_top_compradores(5)
        }

        /// Obtiene los productos más vendidos.
        /// Retorna un vector de tuplas (producto_id, cantidad_ventas).
        #[ink(message)]
        pub fn productos_mas_vendidos(&self) -> Vec<(u128, u32)> {
            self._obtener_productos_mas_vendidos()
        }

        /// Obtiene estadísticas por categoría.
        /// Retorna un vector de tuplas (categoria, total_ventas, calificacion_promedio).
        #[ink(message)]
        pub fn estadisticas_por_categoria(&self) -> Vec<(String, u32, Option<u128>)> {
            self._obtener_estadisticas_categorias()
        }

        /// Obtiene la cantidad de órdenes de un usuario específico.
        #[ink(message)]
        pub fn cantidad_ordenes_usuario(&self, usuario: AccountId) -> u32 {
            self._llamar_marketplace_cantidad_ordenes(usuario)
        }

        // ===== Funciones privadas =====

        /// Hace una llamada cross-contract al marketplace para obtener cantidad de órdenes.
        fn _llamar_marketplace_cantidad_ordenes(&self, usuario: AccountId) -> u32 {
            build_call::<ink::env::DefaultEnvironment>()
                .call(self.marketplace)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "cantidad_ordenes_usuario"
                    )))
                    .push_arg(usuario),
                )
                .returns::<u32>()
                .invoke()
                .unwrap_or(0) // En caso de error, retornar 0
        }

        /// Obtiene el top N de vendedores ordenados por reputación.
        fn _obtener_top_vendedores(&self, cantidad: usize) -> Vec<(AccountId, u128)> {
            // Obtener todos los usuarios con reputación del marketplace
            let usuarios = self._llamar_marketplace_usuarios_con_reputacion();
            
            let mut vendedores: Vec<(AccountId, u128)> = usuarios
                .into_iter()
                .filter_map(|(usuario, reputacion)| {
                    reputacion
                        .promedio_vendedor()
                        .map(|promedio| (usuario, promedio))
                })
                .collect();
            
            // Ordenar por reputación descendente
            vendedores.sort_by(|a, b| b.1.cmp(&a.1));
            vendedores.truncate(cantidad);
            vendedores
        }

        /// Obtiene el top N de compradores ordenados por reputación.
        fn _obtener_top_compradores(&self, cantidad: usize) -> Vec<(AccountId, u128)> {
            // Obtener todos los usuarios con reputación del marketplace
            let usuarios = self._llamar_marketplace_usuarios_con_reputacion();
            
            let mut compradores: Vec<(AccountId, u128)> = usuarios
                .into_iter()
                .filter_map(|(usuario, reputacion)| {
                    reputacion
                        .promedio_comprador()
                        .map(|promedio| (usuario, promedio))
                })
                .collect();
            
            // Ordenar por reputación descendente
            compradores.sort_by(|a, b| b.1.cmp(&a.1));
            compradores.truncate(cantidad);
            compradores
        }

        /// Hace una llamada cross-contract al marketplace para obtener usuarios con reputación.
        fn _llamar_marketplace_usuarios_con_reputacion(&self) -> Vec<(AccountId, ReputacionData)> {
            build_call::<ink::env::DefaultEnvironment>()
                .call(self.marketplace)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "obtener_usuarios_con_reputacion"
                ))))
                .returns::<Vec<(AccountId, ReputacionData)>>()
                .invoke()
                .unwrap_or_default() // En caso de error, retornar vector vacío
        }

        /// Obtiene los productos más vendidos ordenados por cantidad de ventas.
        fn _obtener_productos_mas_vendidos(&self) -> Vec<(u128, u32)> {
            let todos_productos = self._llamar_marketplace_ver_todos_productos();
            
            let mut productos_ventas: Vec<(u128, u32)> = todos_productos
                .into_iter()
                .map(|(id, _)| {
                    let ventas = self._llamar_marketplace_ventas_producto(id);
                    (id, ventas)
                })
                .collect();
            
            // Ordenar por ventas (descendente)
            productos_ventas.sort_by(|a, b| b.1.cmp(&a.1));
            productos_ventas
        }

        /// Hace una llamada cross-contract al marketplace para obtener todos los productos.
        fn _llamar_marketplace_ver_todos_productos(&self) -> Vec<(u128, Producto)> {
            build_call::<ink::env::DefaultEnvironment>()
                .call(self.marketplace)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "ver_todos_los_productos"
                ))))
                .returns::<Vec<(u128, Producto)>>()
                .invoke()
                .unwrap_or_default() // En caso de error, retornar vector vacío
        }

        /// Hace una llamada cross-contract al marketplace para obtener ventas de un producto.
        fn _llamar_marketplace_ventas_producto(&self, producto_id: u128) -> u32 {
            build_call::<ink::env::DefaultEnvironment>()
                .call(self.marketplace)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "obtener_ventas_producto"
                    )))
                    .push_arg(producto_id),
                )
                .returns::<u32>()
                .invoke()
                .unwrap_or(0) // En caso de error, retornar 0
        }

        /// Obtiene las estadísticas agrupadas por categoría.
        fn _obtener_estadisticas_categorias(&self) -> Vec<(String, u32, Option<u128>)> {
            let todos_productos = self._llamar_marketplace_ver_todos_productos();
            let mut stats_map: BTreeMap<String, (u32, u128, u32)> = BTreeMap::new();
            
            for (_, producto) in todos_productos {
                if let Some(stats) = self._llamar_marketplace_estadisticas_categoria(
                    producto.categoria.clone()
                ) {
                    let entry = stats_map.entry(producto.categoria).or_insert((0, 0, 0));
                    // Usar unwrap_or para mantener el valor anterior en caso de overflow
                    entry.0 = entry.0.checked_add(stats.0).unwrap_or(entry.0);
                    entry.1 = entry.1.checked_add(stats.1).unwrap_or(entry.1);
                    entry.2 = entry.2.checked_add(stats.2).unwrap_or(entry.2);
                }
            }
            
            stats_map
                .into_iter()
                .map(|(cat, (total_ventas, suma_calif, num_calif))| {
                    let promedio = if num_calif > 0 {
                        suma_calif.checked_div(num_calif as u128)
                    } else {
                        None
                    };
                    (cat, total_ventas, promedio)
                })
                .collect()
        }

        /// Hace una llamada cross-contract al marketplace para obtener estadísticas de categoría.
        fn _llamar_marketplace_estadisticas_categoria(
            &self,
            categoria: String,
        ) -> Option<(u32, u128, u32)> {
            build_call::<ink::env::DefaultEnvironment>()
                .call(self.marketplace)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "obtener_estadisticas_categoria"
                    )))
                    .push_arg(categoria),
                )
                .returns::<Option<(u32, u128, u32)>>()
                .invoke()
                .unwrap_or(None) // En caso de error, retornar None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        fn default_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }

        fn init_reportes_view(marketplace: AccountId) -> ReportesView {
            ReportesView::new(marketplace)
        }

        #[ink::test]
        fn crear_reportes_view_funciona() {
            let accounts = default_accounts();
            let reportes = init_reportes_view(accounts.alice);
            assert_eq!(reportes.obtener_marketplace(), accounts.alice);
        }

        #[ink::test]
        fn actualizar_marketplace_funciona() {
            let accounts = default_accounts();
            let mut reportes = init_reportes_view(accounts.alice);
            assert_eq!(reportes.obtener_marketplace(), accounts.alice);
            
            reportes.actualizar_marketplace(accounts.bob);
            assert_eq!(reportes.obtener_marketplace(), accounts.bob);
        }

        #[ink::test]
        fn cantidad_ordenes_usuario_retorna_cero_si_no_hay_marketplace() {
            let accounts = default_accounts();
            // Usar una cuenta que no es un contrato desplegado
            let reportes = init_reportes_view(accounts.charlie);
            // Debería retornar 0 si la llamada falla
            let cantidad = reportes.cantidad_ordenes_usuario(accounts.alice);
            assert_eq!(cantidad, 0);
        }

        #[ink::test]
        fn top_vendedores_retorna_vacio_si_no_hay_marketplace() {
            let accounts = default_accounts();
            let reportes = init_reportes_view(accounts.charlie);
            // Debería retornar vacío si la llamada falla
            let top = reportes.top_5_vendedores();
            assert_eq!(top.len(), 0);
        }

        #[ink::test]
        fn top_compradores_retorna_vacio_si_no_hay_marketplace() {
            let accounts = default_accounts();
            let reportes = init_reportes_view(accounts.charlie);
            // Debería retornar vacío si la llamada falla
            let top = reportes.top_5_compradores();
            assert_eq!(top.len(), 0);
        }

        #[ink::test]
        fn productos_mas_vendidos_retorna_vacio_si_no_hay_marketplace() {
            let accounts = default_accounts();
            let reportes = init_reportes_view(accounts.charlie);
            // Debería retornar vacío si la llamada falla
            let productos = reportes.productos_mas_vendidos();
            assert_eq!(productos.len(), 0);
        }

        #[ink::test]
        fn estadisticas_por_categoria_retorna_vacio_si_no_hay_marketplace() {
            let accounts = default_accounts();
            let reportes = init_reportes_view(accounts.charlie);
            // Debería retornar vacío si la llamada falla
            let stats = reportes.estadisticas_por_categoria();
            assert_eq!(stats.len(), 0);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        
        // Nota: Los tests E2E requieren que ambos contratos estén compilados
        // y desplegados. Ver README.md para instrucciones de despliegue en testnet.
        
        // Ejemplo de flujo para testnet:
        // 1. Compilar y desplegar el contrato Marketplace
        // 2. Obtener el AccountId del Marketplace desplegado
        // 3. Compilar y desplegar ReportesView pasando el AccountId del Marketplace
        // 4. Usar ReportesView para hacer consultas al Marketplace
    }
}

